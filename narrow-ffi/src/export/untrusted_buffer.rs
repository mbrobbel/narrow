//! Export of storage whose collections lend a different slice than the one
//! narrow validated, written entirely in safe code.

#![forbid(unsafe_code)]

extern crate std;

use std::{
    borrow::Borrow,
    iter::Copied,
    slice,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    vec,
    vec::Vec,
};

use narrow::{buffer::Buffer, collection::Collection, fixed_size::FixedSize, length::Length};

/// Safe [`Buffer`] whose collections are [`Swap`]s.
#[derive(Default)]
struct Swapping;

impl Buffer for Swapping {
    type For<T: FixedSize> = Swap<T>;
}

/// State shared by the [`Swap`] collections of an array, which arms them all
/// at once and counts their borrows.
#[derive(Default)]
struct State {
    /// Whether the collections lend their swapped items.
    armed: AtomicBool,
    /// Number of borrows since the state was armed.
    borrows: AtomicUsize,
}

/// Collection that behaves honestly until its [`State`] is armed, and then
/// lends a different slice and may report a different length.
///
/// It models what a safe [`Buffer`] implementation can do: narrow validates
/// an array built from `Swap`s against `items`, and the test then arms the
/// state before exporting, so export sees `swapped` and the shrunk length.
/// Export must check what it borrows rather than what narrow validated.
struct Swap<T> {
    /// Items lent, iterated and counted while the state is not armed.
    items: Vec<T>,
    /// Items lent once the state is armed.
    swapped: Vec<T>,
    /// Length reported once armed, after `honest_lengths` armed calls, or
    /// [`None`] to always report the length of `items`.
    shrunk: Option<usize>,
    /// Number of armed calls to [`Length::len`] that still report the length
    /// of `items`.
    honest_lengths: usize,
    /// Number of calls to [`Length::len`] since the state was armed.
    armed_lengths: AtomicUsize,
    /// State shared with the other collections of the array.
    state: Arc<State>,
}

impl<T> Swap<T> {
    /// Lends `items` until `state` is armed, and `swapped` afterwards.
    fn new(items: Vec<T>, swapped: Vec<T>, state: &Arc<State>) -> Self {
        Self {
            items,
            swapped,
            shrunk: None,
            honest_lengths: 0,
            armed_lengths: AtomicUsize::new(0),
            state: Arc::clone(state),
        }
    }
}

impl<T: Clone> Swap<T> {
    /// Lends `items`, but reports `shrunk` as its length once armed and
    /// called `honest_lengths` times.
    fn shrinking(items: Vec<T>, shrunk: usize, honest_lengths: usize, state: &Arc<State>) -> Self {
        Self {
            shrunk: Some(shrunk),
            honest_lengths,
            ..Self::new(items.clone(), items, state)
        }
    }
}

impl<T> Length for Swap<T> {
    fn len(&self) -> usize {
        if !self.state.armed.load(Ordering::Relaxed) {
            return self.items.len();
        }
        let call = self.armed_lengths.fetch_add(1, Ordering::Relaxed);
        self.shrunk
            .filter(|_| call >= self.honest_lengths)
            .unwrap_or(self.items.len())
    }
}

impl<T> Borrow<[T]> for Swap<T> {
    fn borrow(&self) -> &[T] {
        if self.state.armed.load(Ordering::Relaxed) {
            self.state.borrows.fetch_add(1, Ordering::Relaxed);
            &self.swapped
        } else {
            &self.items
        }
    }
}

impl<T: FixedSize> Collection for Swap<T> {
    type View<'collection>
        = T
    where
        Self: 'collection;
    type Owned = T;

    fn view(&self, index: usize) -> Option<T> {
        self.items.get(index).copied()
    }

    type Iter<'collection>
        = Copied<slice::Iter<'collection, T>>
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        self.items.iter().copied()
    }

    type IntoIter = vec::IntoIter<T>;

    fn into_iter_owned(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::{
        sync::{Arc, atomic::Ordering},
        vec::Vec,
    };

    use narrow::{
        array::Array,
        bitmap::Bitmap,
        collection::flatten::Flatten,
        layout::{
            boolean::Boolean, fixed_size_list::FixedSizeList,
            fixed_size_primitive::FixedSizePrimitive, variable_size_list::VariableSizeList,
        },
        nullability::NonNullable,
        offset::{Offset, Offsets},
        validity::Validity,
    };

    use super::super::{ArrowArrayLayout, BufferKind, Export, ExportError, read_list_i32};
    use super::{State, Swap, Swapping};

    /// Arms `state` and exports `array`, returning the exported null count.
    fn export_armed(array: impl Export, state: &State) -> Result<i64, ExportError> {
        state.armed.store(true, Ordering::Relaxed);
        array
            .export()
            .map(|(exported, _schema)| exported.null_count)
    }

    #[test]
    fn rejects_short_values() {
        let state = Arc::default();
        let values = Swap::new(std::vec![1, 2, 3], std::vec![1], &state);
        let array = Array::<i32, Swapping>::from_buffer(FixedSizePrimitive::from_buffer(values));

        assert_eq!(
            export_armed(array, &state),
            Err(ExportError::BufferTooShort {
                buffer: BufferKind::Values,
                required: 3,
                actual: 1,
            })
        );
    }

    #[test]
    fn exports_honest_values() {
        let state = Arc::default();
        let values = Swap::new(std::vec![1, 2, 3], std::vec![4, 5, 6], &state);
        let array = Array::<i32, Swapping>::from_buffer(FixedSizePrimitive::from_buffer(values));

        assert_eq!(export_armed(array, &state), Ok(0));
    }

    #[test]
    fn rejects_short_boolean_values() {
        let state = Arc::default();
        let bits = Swap::new(std::vec![0xff, 0x01], std::vec![0xff], &state);
        let bitmap = Bitmap::<Swapping>::try_from_parts(bits, 9, 0).expect("valid bitmap");
        let array = Array::<bool, Swapping>::from_buffer(Boolean::from_buffer(bitmap));

        assert_eq!(
            export_armed(array, &state),
            Err(ExportError::BufferTooShort {
                buffer: BufferKind::Values,
                required: 9,
                actual: 8,
            })
        );
    }

    /// Returns a validity bitmap of `bits` bits that lends `swapped` once
    /// armed.
    fn validity(
        state: &Arc<State>,
        bits: usize,
        items: Vec<u8>,
        swapped: Vec<u8>,
    ) -> Bitmap<Swapping> {
        Bitmap::try_from_parts(Swap::new(items, swapped, state), bits, 0).expect("valid bitmap")
    }

    #[test]
    fn rejects_short_validity() {
        let state = Arc::default();
        let values = Swap::new(std::vec![1, 2, 3], std::vec![1, 2, 3], &state);
        let bitmap = validity(&state, 3, std::vec![0b0000_0101], std::vec![]);
        let validity = Validity::try_from_parts(values, bitmap).expect("valid parts");
        let array =
            Array::<Option<i32>, Swapping>::from_buffer(FixedSizePrimitive::from_buffer(validity));

        assert_eq!(
            export_armed(array, &state),
            Err(ExportError::BufferTooShort {
                buffer: BufferKind::Validity,
                required: 3,
                actual: 0,
            })
        );
    }

    #[test]
    fn counts_nulls_in_exported_validity() {
        let state = Arc::default();
        let values = Swap::new(std::vec![1, 2, 3], std::vec![1, 2, 3], &state);
        let bitmap = validity(&state, 3, std::vec![0b0000_0111], std::vec![0b1111_1000]);
        let validity = Validity::try_from_parts(values, bitmap).expect("valid parts");
        let array =
            Array::<Option<i32>, Swapping>::from_buffer(FixedSizePrimitive::from_buffer(validity));

        assert_eq!(export_armed(array, &state), Ok(3));
    }

    /// Offsets of the lists `[1]` and `[2]` that lend `swapped` once armed.
    fn offsets<OffsetItem: Offset>(
        state: &Arc<State>,
        swapped: Vec<OffsetItem>,
    ) -> Offsets<FixedSizePrimitive<i32, NonNullable, Swapping>, OffsetItem, Swapping> {
        let values = Swap::new(std::vec![1, 2], std::vec![1, 2], state);
        let items = [0, 1, 2]
            .into_iter()
            .map(|offset| OffsetItem::try_from(offset).expect("small offset"))
            .collect();
        let offset_storage = Swap::new(items, swapped, state);
        Offsets::try_from_parts(FixedSizePrimitive::from_buffer(values), offset_storage)
            .expect("valid offsets")
    }

    /// Exports the lists `[1]` and `[2]` with offsets that lend `swapped`
    /// once armed.
    fn list(swapped: Vec<i32>) -> Result<i64, ExportError> {
        let state = Arc::default();
        let offsets = offsets(&state, swapped);
        let array =
            Array::<Vec<i32>, Swapping>::from_buffer(VariableSizeList::from_buffer(offsets));
        export_armed(array, &state)
    }

    #[test]
    fn rejects_invalid_list_offsets() {
        assert_eq!(list(std::vec![0, 1, 2]), Ok(0));
        assert_eq!(list(std::vec![0, 1, 2, 7]), Ok(0));
        assert_eq!(
            list(std::vec![0, 2]),
            Err(ExportError::BufferTooShort {
                buffer: BufferKind::Offsets,
                required: 3,
                actual: 2,
            })
        );
        assert_eq!(
            list(std::vec![0, 1, 3]),
            Err(ExportError::ChildTooShort {
                required: 3,
                actual: 2,
            })
        );
        assert_eq!(
            list(std::vec![0, 3, 2]),
            Err(ExportError::InvalidOffsets { index: 2 })
        );
        assert_eq!(
            list(std::vec![0, 2, 1]),
            Err(ExportError::InvalidOffsets { index: 2 })
        );
        assert_eq!(
            list(std::vec![-1, 0, 2]),
            Err(ExportError::InvalidOffsets { index: 0 })
        );
        assert_eq!(
            list(std::vec![-1, -2, 2]),
            Err(ExportError::InvalidOffsets { index: 0 })
        );
    }

    #[test]
    fn rejects_large_list_offsets_beyond_child() {
        let state = Arc::default();
        let offsets = offsets(&state, std::vec![0_i64, 1, i64::from(u32::MAX) + 8]);
        let layout = VariableSizeList::<i32, NonNullable, i64, Swapping>::from_buffer(offsets);
        state.armed.store(true, Ordering::Relaxed);

        assert_eq!(
            ArrowArrayLayout::export(layout).map(|_| ()),
            Err(ExportError::ChildTooShort {
                required: usize::try_from(i64::from(u32::MAX) + 8).unwrap_or(usize::MAX),
                actual: 2,
            })
        );
    }

    #[test]
    fn rejects_missing_offsets_of_empty_list() {
        let state = Arc::default();
        let values = Swap::new(std::vec![], std::vec![], &state);
        let offset_storage = Swap::new(std::vec![0], std::vec![], &state);
        let offsets =
            Offsets::try_from_parts(FixedSizePrimitive::from_buffer(values), offset_storage)
                .expect("valid offsets");
        let array =
            Array::<Vec<i32>, Swapping>::from_buffer(VariableSizeList::from_buffer(offsets));

        assert_eq!(
            export_armed(array, &state),
            Err(ExportError::BufferTooShort {
                buffer: BufferKind::Offsets,
                required: 1,
                actual: 0,
            })
        );
    }

    #[test]
    fn rejects_invalid_nullable_list_offsets() {
        let state = Arc::default();
        let offsets = offsets(&state, std::vec![0, 2, 1]);
        let bitmap = validity(&state, 2, std::vec![0b0000_0011], std::vec![0b0000_0011]);
        let validity = Validity::try_from_parts(offsets, bitmap).expect("valid parts");
        let array = Array::<Option<Vec<i32>>, Swapping>::from_buffer(
            VariableSizeList::from_buffer(validity),
        );

        assert_eq!(
            export_armed(array, &state),
            Err(ExportError::InvalidOffsets { index: 2 })
        );
    }

    #[test]
    fn rejects_fixed_size_list_child_that_shrinks() {
        let state = Arc::default();
        let values = Swap::shrinking(std::vec![1, 2, 3, 4], 2, 1, &state);
        let flattened =
            Flatten::try_from_parts(FixedSizePrimitive::from_buffer(values)).expect("valid parts");
        let array = Array::<[i32; 2], Swapping>::from_buffer(FixedSizeList::from_buffer(flattened));

        assert_eq!(
            export_armed(array, &state),
            Err(ExportError::ChildTooShort {
                required: 4,
                actual: 2,
            })
        );
    }

    #[test]
    fn rejects_nullable_fixed_size_list_child_that_shrinks() {
        let state = Arc::default();
        let values = Swap::shrinking(std::vec![1, 2, 3, 4, 5, 6], 2, 0, &state);
        let flattened =
            Flatten::try_from_parts(FixedSizePrimitive::from_buffer(values)).expect("valid parts");
        let bitmap = validity(&state, 3, std::vec![0b0000_0101], std::vec![0b0000_0101]);
        let validity = Validity::try_from_parts(flattened, bitmap).expect("valid parts");
        let array =
            Array::<Option<[i32; 2]>, Swapping>::from_buffer(FixedSizeList::from_buffer(validity));

        assert_eq!(
            export_armed(array, &state),
            Err(ExportError::ChildTooShort {
                required: 6,
                actual: 2,
            })
        );
    }

    #[test]
    fn rejects_list_child_that_shrinks() {
        let state = Arc::default();
        let values = Swap::shrinking(std::vec![1, 2], 1, 0, &state);
        let offset_storage = Swap::new(std::vec![0, 1, 2], std::vec![0, 1, 2], &state);
        let offsets =
            Offsets::try_from_parts(FixedSizePrimitive::from_buffer(values), offset_storage)
                .expect("valid offsets");
        let array =
            Array::<Vec<i32>, Swapping>::from_buffer(VariableSizeList::from_buffer(offsets));

        assert_eq!(
            export_armed(array, &state),
            Err(ExportError::ChildTooShort {
                required: 2,
                actual: 1,
            })
        );
    }

    /// Exports the nullable booleans `[false, None, true]` with validity and
    /// values that lend the given bytes once armed.
    fn nullable_boolean(
        validity_swapped: Vec<u8>,
        values_swapped: Vec<u8>,
    ) -> Result<i64, ExportError> {
        let state = Arc::default();
        let values = validity(&state, 3, std::vec![0b0000_0110], values_swapped);
        let bitmap = validity(&state, 3, std::vec![0b0000_0101], validity_swapped);
        let validity = Validity::try_from_parts(values, bitmap).expect("valid parts");
        let array = Array::<Option<bool>, Swapping>::from_buffer(Boolean::from_buffer(validity));
        export_armed(array, &state)
    }

    #[test]
    fn rejects_short_nullable_boolean_buffers() {
        assert_eq!(
            nullable_boolean(std::vec![0b0000_0101], std::vec![0b0000_0110]),
            Ok(1)
        );
        assert_eq!(
            nullable_boolean(std::vec![], std::vec![0b0000_0110]),
            Err(ExportError::BufferTooShort {
                buffer: BufferKind::Validity,
                required: 3,
                actual: 0,
            })
        );
        assert_eq!(
            nullable_boolean(std::vec![0b0000_0101], std::vec![]),
            Err(ExportError::BufferTooShort {
                buffer: BufferKind::Values,
                required: 3,
                actual: 0,
            })
        );
    }

    #[test]
    fn borrows_each_buffer_once() {
        let state = Arc::default();
        let offsets = offsets(&state, std::vec![0, 1, 2]);
        let bitmap = validity(&state, 2, std::vec![0b0000_0001], std::vec![0b0000_0001]);
        let validity = Validity::try_from_parts(offsets, bitmap).expect("valid parts");
        let array = Array::<Option<Vec<i32>>, Swapping>::from_buffer(
            VariableSizeList::from_buffer(validity),
        );

        assert_eq!(export_armed(array, &state), Ok(1));
        assert_eq!(state.borrows.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn exports_longer_buffers_within_bounds() {
        let state = Arc::default();
        let values = Swap::new(std::vec![1, 2], std::vec![5, 6, 7], &state);
        let offset_storage = Swap::new(std::vec![0, 1, 2], std::vec![0, 1, 2, 9], &state);
        let offsets =
            Offsets::try_from_parts(FixedSizePrimitive::from_buffer(values), offset_storage)
                .expect("valid offsets");
        let array =
            Array::<Vec<i32>, Swapping>::from_buffer(VariableSizeList::from_buffer(offsets));
        state.armed.store(true, Ordering::Relaxed);

        let (exported, _schema) = array.export().expect("export array");

        assert_eq!(exported.length, 2);
        assert_eq!(read_list_i32(&exported), [std::vec![5], std::vec![6]]);
    }
}
