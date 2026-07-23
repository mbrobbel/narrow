//! Borrowed import support for [`FixedSizePrimitive`].

use core::{ffi::CStr, mem, slice};

use narrow::{
    buffer::SliceBuffer, fixed_size::FixedSize, layout::fixed_size_primitive::FixedSizePrimitive,
    nullability::NonNullable,
};

use crate::{ArrowArray, ArrowSchema, ArrowType};

use super::{ImportError, ImportLayout};

impl<'array, T> ImportLayout<'array> for FixedSizePrimitive<T, NonNullable, SliceBuffer<'array>>
where
    T: FixedSize + ArrowType,
{
    const N_BUFFERS: i64 = 2;
    const N_CHILDREN: i64 = 0;

    fn matches_format(format: &CStr) -> bool {
        format == T::FORMAT
    }

    unsafe fn import_validated(
        array: &'array ArrowArray,
        _schema: &ArrowSchema,
        length: usize,
    ) -> Result<Self, ImportError> {
        // SAFETY: The caller guarantees a valid two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        let values = if length == 0 {
            &[]
        } else {
            let values = buffers[1].cast::<T>();
            if values.is_null() {
                return Err(ImportError::MissingValuesBuffer);
            }
            if !values.is_aligned() {
                return Err(ImportError::MisalignedValuesBuffer {
                    alignment: mem::align_of::<T>(),
                });
            }
            // SAFETY: The caller guarantees the value buffer contains `length`
            // properly aligned values that remain immutable for `'array`.
            unsafe { slice::from_raw_parts(values, length) }
        };

        Ok(Self::from_buffer(values))
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::sync::Arc;
    use core::borrow::Borrow;

    use narrow::{
        array::Array,
        buffer::{ArcBuffer, BufferRef, SliceBuffer},
        layout::fixed_size_primitive::FixedSizePrimitive,
    };

    use crate::{
        export::Export,
        import::{Import, ImportError},
    };

    #[test]
    fn imports_primitive_values_without_copying() {
        let storage = Arc::<[i32]>::from([1, 2, 3]);
        let weak = Arc::downgrade(&storage);
        let data = storage.as_ptr();
        let source: Array<i32, ArcBuffer> =
            Array::from_buffer(FixedSizePrimitive::from_buffer(storage));
        let (array, schema) = source.export().expect("export array");

        {
            // SAFETY: The exported structures remain live and own a valid i32
            // buffer for the lifetime of the imported array.
            let imported: Array<i32, SliceBuffer<'_>> =
                unsafe { Import::import(&array, &schema) }.expect("import array");

            let imported_values: &[i32] = imported.buffer_ref().buffer_ref().borrow();
            assert_eq!(imported_values, [1, 2, 3]);
            assert_eq!(imported_values.as_ptr(), data);
            assert!(weak.upgrade().is_some());
        };

        assert!(weak.upgrade().is_some());
        drop(array);
        assert!(weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn rejects_mismatched_primitive_format() {
        let source = [1_i32].into_iter().collect::<Array<i32>>();
        let (array, mut schema) = source.export().expect("export array");
        schema.format = c"l".as_ptr();

        // SAFETY: The exported structures and value buffer remain valid; only
        // the schema format is changed to exercise validation.
        let error = unsafe {
            <Array<i32, SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect_err("mismatched format")
        };

        assert_eq!(error, ImportError::UnexpectedFormat);
    }

    #[test]
    fn rejects_non_zero_primitive_offset() {
        let source = [1_i32].into_iter().collect::<Array<i32>>();
        let (mut array, schema) = source.export().expect("export array");
        array.offset = 1;

        // SAFETY: The exported structures and value buffer remain valid; only
        // the array offset is changed to exercise validation.
        let error = unsafe {
            <Array<i32, SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect_err("non-zero offset")
        };

        assert_eq!(error, ImportError::NonZeroOffset { offset: 1 });
    }
}
