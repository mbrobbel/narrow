//! Borrowed import support for [`Boolean`].

use core::{ffi::CStr, slice};

use narrow::{bitmap::Bitmap, buffer::SliceBuffer, layout::boolean::Boolean};

use crate::{ArrowArray, ArrowType};

use super::{ImportError, ImportLayout, ImportNullability};

impl<'array, Nulls> ImportLayout<'array> for Boolean<Nulls, SliceBuffer<'array>>
where
    Nulls: ImportNullability<'array>,
{
    const FLAGS: i64 = Nulls::FLAGS;
    const BUFFERS: i64 = 2;
    const CHILDREN: i64 = 0;

    fn matches_format(format: &CStr) -> bool {
        format == bool::FORMAT
    }

    unsafe fn import_validated(
        array: &'array ArrowArray,
        length: usize,
    ) -> Result<Self, ImportError> {
        // SAFETY: The caller guarantees a valid two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        let byte_length = length.div_ceil(8);
        let values = if byte_length == 0 {
            &[]
        } else {
            let values = buffers[1].cast::<u8>();
            if values.is_null() {
                return Err(ImportError::MissingValuesBuffer);
            }
            // SAFETY: The caller guarantees the value buffer contains
            // `byte_length` bytes that remain immutable for `'array`.
            unsafe { slice::from_raw_parts(values, byte_length) }
        };
        let bitmap = Bitmap::try_from_parts(values, length, 0)
            .expect("imported Boolean buffer contains the declared number of bits");

        // SAFETY: Common validation and the caller guarantee a valid optional
        // validity buffer for the imported values.
        let collection = unsafe { Nulls::wrap(array, bitmap) }?;
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
        layout::boolean::Boolean,
        validity::Validity,
    };

    use crate::{
        export::Export,
        import::{Import, ImportError},
    };

    #[test]
    fn imports_boolean_values_without_copying() {
        let storage = Arc::<[u8]>::from([0b0000_0101]);
        let weak = Arc::downgrade(&storage);
        let data = storage.as_ptr();
        let bitmap = Bitmap::<ArcBuffer>::try_from_parts(storage, 3, 0).expect("valid bitmap");
        let source: Array<bool, ArcBuffer> = Array::from_buffer(Boolean::from_buffer(bitmap));
        let (array, schema) = source.export().expect("export array");

        {
            // SAFETY: The exported structures remain live and own a valid
            // Boolean value bitmap for the lifetime of the imported array.
            let imported: Array<bool, SliceBuffer<'_>> =
                unsafe { Import::import(&array, &schema) }.expect("import array");

            let imported_values: &[u8] = imported.buffer_ref().buffer_ref().buffer_ref().borrow();
            assert_eq!(imported_values, [0b0000_0101]);
            assert_eq!(imported_values.as_ptr(), data);
            assert_eq!(
                imported.iter_views().collect::<alloc::vec::Vec<_>>(),
                [true, false, true,]
            );
            assert!(weak.upgrade().is_some());
        };

        assert!(weak.upgrade().is_some());
        drop(array);
        assert!(weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn imports_nullable_boolean_values_without_copying() {
        let value_bytes = Arc::<[u8]>::from([0b0000_0001]);
        let validity_bytes = Arc::<[u8]>::from([0b0000_0101]);
        let values_data = value_bytes.as_ptr();
        let validity_data = validity_bytes.as_ptr();
        let value_bitmap =
            Bitmap::<ArcBuffer>::try_from_parts(value_bytes, 3, 0).expect("valid bitmap");
        let validity =
            Bitmap::<ArcBuffer>::try_from_parts(validity_bytes, 3, 0).expect("valid bitmap");
        let values = Validity::try_from_parts(value_bitmap, validity).expect("valid parts");
        let source: Array<Option<bool>, ArcBuffer> =
            Array::from_buffer(Boolean::from_buffer(values));
        let (mut array, schema) = source.export().expect("export array");
        array.null_count = -1;

        // SAFETY: The exported structures retain valid value and validity
        // buffers for the lifetime of the imported array.
        let imported: Array<Option<bool>, SliceBuffer<'_>> =
            unsafe { Import::import(&array, &schema) }.expect("import array");

        let imported_validity = imported.buffer_ref().buffer_ref();
        let imported_values: &[u8] = imported_validity.child_ref().buffer_ref().borrow();
        let imported_bitmap = imported_validity.bitmap_ref().expect("explicit validity");
        let imported_validity_values: &[u8] = imported_bitmap.buffer_ref().borrow();
        assert_eq!(imported_values.as_ptr(), values_data);
        assert_eq!(imported_validity_values.as_ptr(), validity_data);
        assert_eq!(
            imported.iter_views().collect::<alloc::vec::Vec<_>>(),
            [Some(true), None, Some(false)]
        );
    }

    #[test]
    fn imports_nullable_boolean_values_with_implicit_validity() {
        let value_bytes = Arc::<[u8]>::from([0b0000_0101]);
        let values_data = value_bytes.as_ptr();
        let value_bitmap =
            Bitmap::<ArcBuffer>::try_from_parts(value_bytes, 3, 0).expect("valid bitmap");
        let values = Validity::<_, ArcBuffer>::from_collection(value_bitmap);
        let source: Array<Option<bool>, ArcBuffer> =
            Array::from_buffer(Boolean::from_buffer(values));
        let (array, schema) = source.export().expect("export array");

        // SAFETY: The exported structures retain a valid value buffer for the
        // lifetime of the imported array.
        let imported: Array<Option<bool>, SliceBuffer<'_>> =
            unsafe { Import::import(&array, &schema) }.expect("import array");

        let imported_validity = imported.buffer_ref().buffer_ref();
        let imported_values: &[u8] = imported_validity.child_ref().buffer_ref().borrow();
        assert_eq!(imported_values.as_ptr(), values_data);
        assert!(imported_validity.bitmap_ref().is_none());
        assert_eq!(
            imported.iter_views().collect::<alloc::vec::Vec<_>>(),
            [Some(true), Some(false), Some(true)]
        );
    }

    #[test]
    fn rejects_missing_nullable_boolean_validity() {
        let source = [Some(true), None]
            .into_iter()
            .collect::<Array<Option<bool>>>();
        let (array, schema) = source.export().expect("export array");
        // SAFETY: The exported array owns a writable two-entry buffer pointer
        // array. Clearing its validity entry exercises import validation.
        unsafe { *array.buffers = ptr::null() };

        // SAFETY: The value buffer remains valid, and the missing validity
        // buffer is the condition under test.
        let error = unsafe {
            <Array<Option<bool>, SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect_err("missing validity")
        };
        assert_eq!(error, ImportError::MissingValidityBuffer);
    }

    #[test]
    fn rejects_nullable_boolean_null_count_mismatch() {
        let source = [Some(true), None]
            .into_iter()
            .collect::<Array<Option<bool>>>();
        let (mut array, schema) = source.export().expect("export array");
        array.null_count = 0;

        // SAFETY: The exported buffers remain valid; only the null count is
        // changed to exercise validation.
        let error = unsafe {
            <Array<Option<bool>, SliceBuffer<'_>> as Import>::import(&array, &schema)
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
