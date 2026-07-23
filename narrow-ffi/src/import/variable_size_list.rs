//! Borrowed import support for [`VariableSizeList`].

use core::{ffi::CStr, mem, slice};

use narrow::{
    buffer::SliceBuffer,
    layout::{ArrayItem, variable_size_list::VariableSizeList},
    nullability::NonNullable,
    offset::Offsets,
};

use crate::{ArrowArray, ArrowListOffset, ArrowSchema};

use super::{ImportError, ImportLayout};

impl<'array, T, OffsetItem> ImportLayout<'array>
    for VariableSizeList<T, NonNullable, OffsetItem, SliceBuffer<'array>>
where
    T: ArrayItem,
    OffsetItem: ArrowListOffset,
    T::Memory<SliceBuffer<'array>>: ImportLayout<'array>,
{
    const BUFFERS: i64 = 2;
    const CHILDREN: i64 = 1;

    fn matches_format(format: &CStr) -> bool {
        format == OffsetItem::FORMAT
    }

    unsafe fn import_validated(
        array: &'array ArrowArray,
        schema: &ArrowSchema,
        length: usize,
    ) -> Result<Self, ImportError> {
        let offsets_length = length.checked_add(1).ok_or(ImportError::InvalidLength {
            length: array.length,
        })?;
        // SAFETY: Common validation guarantees a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        let offsets_pointer = buffers[1].cast::<OffsetItem>();
        if offsets_pointer.is_null() {
            return Err(ImportError::MissingOffsetsBuffer);
        }
        if !offsets_pointer.is_aligned() {
            return Err(ImportError::MisalignedOffsetsBuffer {
                alignment: mem::align_of::<OffsetItem>(),
            });
        }
        // SAFETY: The caller guarantees the offsets buffer contains
        // `offsets_length` aligned values that remain immutable for `'array`.
        let offset_values = unsafe { slice::from_raw_parts(offsets_pointer, offsets_length) };

        // SAFETY: Common parent fields are validated and the caller upholds
        // the Arrow C Data requirements for the retained child structures.
        let child =
            unsafe { Self::import_child::<T::Memory<SliceBuffer<'array>>>(array, schema, 0) }?;
        let offsets = Offsets::try_from_parts(child, offset_values)
            .map_err(|error| ImportError::InvalidOffsets { error })?;

        Ok(Self::from_buffer(offsets))
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::{sync::Arc, vec, vec::Vec};
    use core::borrow::Borrow;

    use narrow::{
        array::Array,
        buffer::{ArcBuffer, BufferRef, SliceBuffer},
        collection::{ChildRef, Collection},
        layout::{fixed_size_primitive::FixedSizePrimitive, variable_size_list::VariableSizeList},
        offset::{Offsets, OffsetsError},
    };

    use crate::{
        export::Export,
        import::{Import, ImportError},
    };

    #[test]
    fn imports_variable_size_list_buffers_without_copying() {
        let value_storage = Arc::<[i32]>::from([1, 2, 3]);
        let offset_storage = Arc::<[i32]>::from([0, 2, 3]);
        let values_weak = Arc::downgrade(&value_storage);
        let offsets_weak = Arc::downgrade(&offset_storage);
        let values_data = value_storage.as_ptr();
        let offsets_data = offset_storage.as_ptr();
        let values = FixedSizePrimitive::from_buffer(value_storage);
        let offsets = Offsets::try_from_parts(values, offset_storage).expect("valid offsets");
        let source: Array<Vec<i32>, ArcBuffer> =
            Array::from_buffer(VariableSizeList::from_buffer(offsets));
        let (array, schema) = source.export().expect("export array");

        {
            // SAFETY: The exported structures remain live and retain their
            // offsets, child array, and values for the imported array's lifetime.
            let imported: Array<Vec<i32>, SliceBuffer<'_>> =
                unsafe { Import::import(&array, &schema) }.expect("import array");

            let imported_offsets = imported.buffer_ref().buffer_ref();
            let offset_values: &[i32] = imported_offsets.buffer_ref().borrow();
            let imported_values: &[i32] = imported_offsets.child_ref().buffer_ref().borrow();
            assert_eq!(offset_values, [0, 2, 3]);
            assert_eq!(offset_values.as_ptr(), offsets_data);
            assert_eq!(imported_values, [1, 2, 3]);
            assert_eq!(imported_values.as_ptr(), values_data);
            assert_eq!(imported.owned(0), Some(vec![1, 2]));
            assert_eq!(imported.owned(1), Some(vec![3]));
            assert!(values_weak.upgrade().is_some());
            assert!(offsets_weak.upgrade().is_some());
        };

        assert!(values_weak.upgrade().is_some());
        assert!(offsets_weak.upgrade().is_some());
        drop(array);
        assert!(values_weak.upgrade().is_none());
        assert!(offsets_weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn rejects_mismatched_variable_size_list_format() {
        let source = [vec![1_i32]].into_iter().collect::<Array<Vec<i32>>>();
        let (array, mut schema) = source.export().expect("export array");
        schema.format = c"+L".as_ptr();

        // SAFETY: The exported structures and buffers remain valid; only the
        // parent format is changed to exercise list format validation.
        let error = unsafe {
            <Array<Vec<i32>, SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect_err("mismatched variable-size-list format")
        };

        assert_eq!(error, ImportError::UnexpectedFormat);
    }

    #[test]
    fn rejects_non_monotonic_variable_size_list_offsets() {
        let source = [vec![1_i32, 2], vec![3]]
            .into_iter()
            .collect::<Array<Vec<i32>>>();
        let (array, schema) = source.export().expect("export array");
        let invalid_offsets = [0_i32, 2, 1];
        // SAFETY: The exported array owns a writable two-entry pointer array;
        // the second entry is the offsets buffer.
        let offsets_slot = unsafe { array.buffers.add(1) };
        // SAFETY: The slot is live and writable, and the replacement buffer
        // remains live through the import call.
        unsafe { *offsets_slot = invalid_offsets.as_ptr().cast() };

        // SAFETY: Every referenced pointer remains valid and sufficiently
        // sized; the malformed offset values are validated by the importer.
        let error = unsafe {
            <Array<Vec<i32>, SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect_err("non-monotonic offsets")
        };

        assert_eq!(
            error,
            ImportError::InvalidOffsets {
                error: OffsetsError::NonMonotonic { index: 2 },
            }
        );
    }
}
