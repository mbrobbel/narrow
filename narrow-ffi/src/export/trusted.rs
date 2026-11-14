//! Storage that export may trust to skip checks that read every item.

use core::{fmt, marker::PhantomData};

use narrow::buffer::{ArcBuffer, ArrayBuffer, BoxBuffer, Buffer, RcBuffer, SliceBuffer, VecBuffer};

/// Proof that the collections of `Storage` borrow as the slices narrow
/// validated and maintains.
///
/// Only narrow-ffi creates these, for the storages provided by
/// [`narrow::buffer`]. The token is specific to `Storage`, so the token of one
/// storage cannot be used for another:
///
/// ```compile_fail,E0308
/// use narrow::{
///     buffer::{Buffer, VecBuffer},
///     fixed_size::FixedSize,
/// };
/// use narrow_ffi::{ExportBuffer, Trusted};
///
/// #[derive(Default)]
/// struct MyBuffer;
///
/// impl Buffer for MyBuffer {
///     type For<T: FixedSize> = Vec<T>;
/// }
///
/// impl ExportBuffer for MyBuffer {
///     const TRUSTED: Option<Trusted<Self>> = VecBuffer::TRUSTED;
/// }
/// ```
pub struct Trusted<Storage: ?Sized>(
    /// Ties the token to `Storage` without owning one.
    PhantomData<fn() -> Storage>,
);

impl<Storage: ?Sized> Trusted<Storage> {
    /// Creates a token for `Storage`.
    ///
    /// # Safety
    ///
    /// For every `T`, `Borrow::<[T]>::borrow` of `Storage::For<T>` must return
    /// the items narrow read when it built, validated or last changed the
    /// collection, for as long as the collection is not accessed through
    /// `&mut`.
    ///
    /// Export then skips checking that list offsets do not decrease, so
    /// `Storage` must also be covered by the
    /// [guarantee](narrow::offset::Offsets#guarantees) that narrow keeps its
    /// offsets sorted.
    const unsafe fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Storage: ?Sized> fmt::Debug for Trusted<Storage> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Trusted")
    }
}

/// [`Buffer`] storage that can be exported.
///
/// Export checks every exported buffer against the array. Storage provided by
/// [`narrow::buffer`] carries a [`Trusted`] token, which lets export skip the
/// one check that reads every item: that list offsets do not decrease, which
/// narrow guarantees for these storages. Debug builds still run it as an
/// assertion. Other storage implements this trait without a token:
///
/// ```
/// use narrow::{
///     array::Array,
///     buffer::{Buffer, EmptyBuffer},
///     fixed_size::FixedSize,
/// };
/// use narrow_ffi::{Export, ExportBuffer};
///
/// #[derive(Default)]
/// struct MyBuffer;
///
/// impl Buffer for MyBuffer {
///     type For<T: FixedSize> = Vec<T>;
/// }
///
/// impl EmptyBuffer for MyBuffer {}
///
/// impl ExportBuffer for MyBuffer {}
///
/// let array = [vec![1], vec![2, 3]]
///     .into_iter()
///     .collect::<Array<Vec<i32>, MyBuffer>>();
/// assert!(array.export().is_ok());
/// ```
pub trait ExportBuffer: Buffer {
    /// Token that lets export skip checking that list offsets do not decrease.
    const TRUSTED: Option<Trusted<Self>> = None;
}

impl ExportBuffer for VecBuffer {
    // SAFETY: A `Vec` borrows as its items, which only change through `&mut`.
    const TRUSTED: Option<Trusted<Self>> = Some(unsafe { Trusted::new() });
}

impl ExportBuffer for BoxBuffer {
    // SAFETY: A boxed slice borrows as its items, which only change through
    // `&mut`.
    const TRUSTED: Option<Trusted<Self>> = Some(unsafe { Trusted::new() });
}

impl ExportBuffer for RcBuffer {
    // SAFETY: A shared slice borrows as its items, and narrow never mutates it.
    const TRUSTED: Option<Trusted<Self>> = Some(unsafe { Trusted::new() });
}

impl ExportBuffer for ArcBuffer {
    // SAFETY: As for `RcBuffer`.
    const TRUSTED: Option<Trusted<Self>> = Some(unsafe { Trusted::new() });
}

impl<const N: usize> ExportBuffer for ArrayBuffer<N> {
    // SAFETY: An array borrows as its items, which only change through `&mut`.
    const TRUSTED: Option<Trusted<Self>> = Some(unsafe { Trusted::new() });
}

impl ExportBuffer for SliceBuffer<'_> {
    // SAFETY: A slice reference borrows as the immutable items it refers to.
    const TRUSTED: Option<Trusted<Self>> = Some(unsafe { Trusted::new() });
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    extern crate std;

    use alloc::{boxed::Box, vec::Vec};
    use core::slice;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use narrow::{
        array::Array,
        buffer::{BoxBuffer, SliceBuffer},
        collection::CollectionRealloc,
        layout::{
            ArrayItem, fixed_size_primitive::FixedSizePrimitive,
            variable_size_list::VariableSizeList,
        },
        length::Length,
        offset::Offsets,
    };

    use crate::{ArrowArray, Export};

    use super::ExportBuffer;

    /// Steps of a deterministic sequence of list operations.
    const STEPS: usize = if cfg!(miri) { 12 } else { 200 };

    /// Returns the offsets of an exported list array.
    fn exported_offsets(array: &ArrowArray) -> &[i32] {
        let length = usize::try_from(array.length).expect("non-negative length");
        // SAFETY: An exported list array has two buffers.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        // SAFETY: The offsets buffer holds `length + 1` offsets.
        unsafe { slice::from_raw_parts(buffers[1].cast::<i32>(), length.strict_add(1)) }
    }

    /// Applies extends, fallible extends, truncates and interrupted extends to
    /// `array`, and checks that the offsets exported after every step are
    /// sorted.
    fn exercise<T>(mut array: Array<T>, item: impl Fn(usize) -> T)
    where
        T: ArrayItem,
        Array<T>: Clone + Extend<T> + Export + CollectionRealloc<Owned = T>,
    {
        let mut state = 0x2545_f491_u32;
        for step in 0..STEPS {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let count = usize::try_from(state % 5).expect("small count");
            match state % 4 {
                0 => array.extend((0..count).map(&item)),
                1 => array
                    .try_extend((0..count).map(&item))
                    .expect("extend array"),
                2 => array.truncate(array.len().saturating_sub(count)),
                _ => {
                    let items = (0..=count).map(|index| {
                        assert!(index < count, "interrupt extend");
                        item(index)
                    });
                    let interrupted = catch_unwind(AssertUnwindSafe(|| {
                        if step % 2 == 0 {
                            array.extend(items);
                        } else {
                            array.try_extend(items).expect("extend array");
                        }
                    }));
                    assert!(interrupted.is_err(), "step {step} was not interrupted");
                }
            }
            let (exported, _schema) = array.clone().export().expect("export array");
            assert!(exported_offsets(&exported).is_sorted(), "step {step}");
        }
    }

    #[test]
    fn exports_lists_of_immutable_trusted_storage() {
        fn list<Storage: ExportBuffer>(array: Array<Vec<i32>, Storage>) -> Vec<i32>
        where
            Array<Vec<i32>, Storage>: Export,
        {
            let (exported, _schema) = array.export().expect("export array");
            exported_offsets(&exported).to_vec()
        }

        let boxed = Offsets::from_parts(
            FixedSizePrimitive::from_buffer(Box::from([1, 2, 3])),
            Box::from([0, 2, 3]),
        );
        let sliced = Offsets::from_parts(
            FixedSizePrimitive::from_buffer(&[1, 2, 3][..]),
            &[0, 2, 3][..],
        );

        assert_eq!(
            list::<BoxBuffer>(Array::from_buffer(VariableSizeList::from_buffer(boxed))),
            [0, 2, 3]
        );
        assert_eq!(
            list::<SliceBuffer<'static>>(Array::from_buffer(VariableSizeList::from_buffer(sliced))),
            [0, 2, 3]
        );
    }

    #[test]
    fn trusted_lists_keep_sorted_offsets() {
        exercise(Array::<Vec<i32>>::default(), |index| {
            (0..index)
                .map(|value| i32::try_from(value).expect("small value"))
                .collect()
        });
        exercise(Array::<Option<Vec<i32>>>::default(), |index| {
            (index % 2 == 0).then(|| Vec::from([1; 3]))
        });
        exercise(Array::<Vec<Vec<i32>>>::default(), |index| {
            (0..index).map(|_| Vec::from([1, 2])).collect()
        });
        exercise(Array::<Vec<Option<[i32; 2]>>>::default(), |index| {
            (0..index)
                .map(|value| (value % 2 == 0).then_some([1, 2]))
                .collect()
        });
    }
}
