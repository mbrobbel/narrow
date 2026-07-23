//! Borrowed import support for [`Boolean`].

use core::{ffi::CStr, slice};

use narrow::{
    bitmap::Bitmap, buffer::SliceBuffer, layout::boolean::Boolean, nullability::NonNullable,
};

use crate::{ArrowArray, ArrowSchema, ArrowType};

use super::{ImportError, ImportLayout};

impl<'array> ImportLayout<'array> for Boolean<NonNullable, SliceBuffer<'array>> {
    const BUFFERS: i64 = 2;
    const CHILDREN: i64 = 0;

    fn matches_format(format: &CStr) -> bool {
        format == bool::FORMAT
    }

    unsafe fn import_validated(
        array: &'array ArrowArray,
        _schema: &ArrowSchema,
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

        Ok(Self::from_buffer(bitmap))
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::sync::Arc;
    use core::borrow::Borrow;

    use narrow::{
        array::Array,
        bitmap::Bitmap,
        buffer::{ArcBuffer, BufferRef, SliceBuffer},
        collection::Collection,
        layout::boolean::Boolean,
    };

    use crate::{export::Export, import::Import};

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
}
