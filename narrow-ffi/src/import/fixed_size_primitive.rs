//! Borrowed import support for [`FixedSizePrimitive`].

use core::{ffi::CStr, mem, slice};

use narrow::{
    buffer::SliceBuffer, fixed_size::FixedSize, layout::fixed_size_primitive::FixedSizePrimitive,
};

use crate::{ArrowArray, ArrowSchema, ArrowType};

use super::{ImportError, ImportLayout, ImportNullability};

impl<'array, T, Nulls> ImportLayout<'array> for FixedSizePrimitive<T, Nulls, SliceBuffer<'array>>
where
    T: FixedSize + ArrowType,
    Nulls: ImportNullability<'array>,
{
    const FLAGS: i64 = Nulls::FLAGS;
    const BUFFERS: i64 = 2;
    const CHILDREN: i64 = 0;

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

        // SAFETY: Common validation and the caller guarantee a valid optional
        // validity buffer for the imported values.
        let collection = unsafe { Nulls::wrap(array, values) }?;
        Ok(Self::from_buffer(collection))
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::sync::Arc;
    use core::{borrow::Borrow, ptr};

    use narrow::{
        array::Array,
        bitmap::{Bitmap, ValidityBitmap},
        buffer::{ArcBuffer, BufferRef, SliceBuffer},
        collection::{ChildRef, Collection},
        layout::fixed_size_primitive::FixedSizePrimitive,
        length::Length,
        validity::Validity,
    };

    use crate::{
        ARROW_FLAG_NULLABLE,
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

    #[test]
    fn ignores_unknown_schema_flags() {
        let source = [1_i32].into_iter().collect::<Array<i32>>();
        let (array, mut schema) = source.export().expect("export array");
        schema.flags = 8;

        // SAFETY: The exported structures and value buffer remain valid; only
        // an unknown schema flag is added to exercise validation.
        let imported = unsafe {
            <Array<i32, SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect("unknown flags are ignored")
        };
        assert_eq!(imported.buffer_ref().len(), 1);
    }

    #[test]
    fn rejects_nullable_schema_flag() {
        let source = [1_i32].into_iter().collect::<Array<i32>>();
        let (array, mut schema) = source.export().expect("export array");
        schema.flags = ARROW_FLAG_NULLABLE;

        // SAFETY: The exported structures and value buffer remain valid; only
        // the schema nullable flag is changed to exercise validation.
        let error = unsafe {
            <Array<i32, SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect_err("nullable flag does not match")
        };
        assert_eq!(
            error,
            ImportError::UnexpectedFlags {
                flags: ARROW_FLAG_NULLABLE
            }
        );
    }

    #[test]
    fn accepts_unknown_null_count() {
        let source = [1_i32].into_iter().collect::<Array<i32>>();
        let (mut array, schema) = source.export().expect("export array");
        array.null_count = -1;

        // SAFETY: The exported structures and value buffer remain valid; only
        // the null count is marked unknown as permitted by Arrow.
        let imported = unsafe {
            <Array<i32, SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect("unknown null count is supported")
        };
        assert_eq!(imported.buffer_ref().len(), 1);
    }

    #[test]
    fn rejects_invalid_negative_null_count() {
        let source = [1_i32].into_iter().collect::<Array<i32>>();
        let (mut array, schema) = source.export().expect("export array");
        array.null_count = -2;

        // SAFETY: The exported structures and value buffer remain valid; only
        // the null count is changed to exercise validation.
        let error = unsafe {
            <Array<i32, SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect_err("invalid negative null count")
        };
        assert_eq!(error, ImportError::UnexpectedNullCount { null_count: -2 });
    }

    #[test]
    fn rejects_known_nulls_for_non_nullable_layout() {
        let source = [1_i32].into_iter().collect::<Array<i32>>();
        let (mut array, schema) = source.export().expect("export array");
        array.null_count = 1;

        // SAFETY: The exported structures and value buffer remain valid; only
        // the null count is changed to exercise validation.
        let error = unsafe {
            <Array<i32, SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect_err("known null does not match")
        };
        assert_eq!(error, ImportError::UnexpectedNullCount { null_count: 1 });
    }

    #[test]
    fn imports_nullable_primitive_values_without_copying() {
        let values = Arc::<[i32]>::from([1, 0, 3]);
        let validity_values = Arc::<[u8]>::from([0b0000_0101]);
        let values_data = values.as_ptr();
        let validity_data = validity_values.as_ptr();
        let bitmap =
            Bitmap::<ArcBuffer>::try_from_parts(validity_values, 3, 0).expect("valid bitmap");
        let validity = Validity::try_from_parts(values, bitmap).expect("valid parts");
        let source: Array<Option<i32>, ArcBuffer> =
            Array::from_buffer(FixedSizePrimitive::from_buffer(validity));
        let (mut array, schema) = source.export().expect("export array");
        array.null_count = -1;

        // SAFETY: The exported structures retain valid value and validity
        // buffers for the lifetime of the imported array.
        let imported: Array<Option<i32>, SliceBuffer<'_>> =
            unsafe { Import::import(&array, &schema) }.expect("import array");

        let imported_validity = imported.buffer_ref().buffer_ref();
        let imported_values: &[i32] = imported_validity.child_ref().borrow();
        let imported_bitmap = imported_validity.bitmap_ref().expect("explicit validity");
        let imported_validity_values: &[u8] = imported_bitmap.buffer_ref().borrow();
        assert_eq!(imported_values.as_ptr(), values_data);
        assert_eq!(imported_validity_values.as_ptr(), validity_data);
        assert_eq!(
            imported.iter_views().collect::<alloc::vec::Vec<_>>(),
            [Some(1), None, Some(3)]
        );
    }

    #[test]
    fn imports_nullable_primitive_values_with_implicit_validity() {
        let values = Arc::<[i32]>::from([1, 2, 3]);
        let data = values.as_ptr();
        let validity = Validity::<_, ArcBuffer>::from_collection(values);
        let source: Array<Option<i32>, ArcBuffer> =
            Array::from_buffer(FixedSizePrimitive::from_buffer(validity));
        let (array, schema) = source.export().expect("export array");

        // SAFETY: The exported structures retain a valid value buffer for the
        // lifetime of the imported array.
        let imported: Array<Option<i32>, SliceBuffer<'_>> =
            unsafe { Import::import(&array, &schema) }.expect("import array");

        let imported_validity = imported.buffer_ref().buffer_ref();
        let imported_values: &[i32] = imported_validity.child_ref().borrow();
        assert_eq!(imported_values.as_ptr(), data);
        assert!(imported_validity.bitmap_ref().is_none());
        assert_eq!(
            imported.iter_views().collect::<alloc::vec::Vec<_>>(),
            [Some(1), Some(2), Some(3)]
        );
    }

    #[test]
    fn rejects_missing_nullable_primitive_validity() {
        let source = [Some(1_i32), None]
            .into_iter()
            .collect::<Array<Option<i32>>>();
        let (array, schema) = source.export().expect("export array");
        // SAFETY: The exported array owns a writable two-entry buffer pointer
        // array. Clearing its validity entry exercises import validation.
        unsafe { *array.buffers = ptr::null() };

        // SAFETY: The value buffer remains valid, and the missing validity
        // buffer is the condition under test.
        let error = unsafe {
            <Array<Option<i32>, SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect_err("missing validity")
        };
        assert_eq!(error, ImportError::MissingValidityBuffer);
    }

    #[test]
    fn rejects_nullable_primitive_null_count_mismatch() {
        let source = [Some(1_i32), None]
            .into_iter()
            .collect::<Array<Option<i32>>>();
        let (mut array, schema) = source.export().expect("export array");
        array.null_count = 0;

        // SAFETY: The exported buffers remain valid; only the null count is
        // changed to exercise validation.
        let error = unsafe {
            <Array<Option<i32>, SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect_err("mismatched null count")
        };
        assert_eq!(
            error,
            ImportError::NullCountMismatch {
                declared: 0,
                actual: 1,
            }
        );
    }
}
