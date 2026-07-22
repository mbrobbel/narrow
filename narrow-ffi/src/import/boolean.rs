//! Borrowed import support for [`Boolean`].

use core::{ffi::CStr, slice};

use narrow::{
    bitmap::Bitmap, buffer::SliceBuffer, layout::boolean::Boolean, nullability::NonNullable,
};

use crate::{ArrowArray, ArrowSchema, ArrowType};

use super::{ArrowArrayImport, ImportError};

impl<'array> ArrowArrayImport<'array> for Boolean<NonNullable, SliceBuffer<'array>> {
    unsafe fn import_layout(
        array: &'array ArrowArray,
        schema: &ArrowSchema,
    ) -> Result<Self, ImportError> {
        if array.is_released() {
            return Err(ImportError::ReleasedArray);
        }
        if schema.is_released() {
            return Err(ImportError::ReleasedSchema);
        }
        if schema.format.is_null() {
            return Err(ImportError::MissingFormat);
        }
        // SAFETY: The caller guarantees a valid null-terminated schema format.
        if unsafe { CStr::from_ptr(schema.format) } != bool::FORMAT {
            return Err(ImportError::UnexpectedFormat);
        }
        if schema.flags != 0 {
            return Err(ImportError::UnexpectedFlags {
                flags: schema.flags,
            });
        }

        let length = usize::try_from(array.length).map_err(|_| ImportError::InvalidLength {
            length: array.length,
        })?;
        if array.offset != 0 {
            return Err(ImportError::NonZeroOffset {
                offset: array.offset,
            });
        }
        if array.null_count != 0 {
            return Err(ImportError::UnexpectedNullCount {
                null_count: array.null_count,
            });
        }
        if array.n_buffers != 2 {
            return Err(ImportError::UnexpectedBufferCount {
                count: array.n_buffers,
            });
        }
        if array.n_children != 0 || schema.n_children != 0 {
            return Err(ImportError::UnexpectedChildCount {
                array: array.n_children,
                schema: schema.n_children,
            });
        }
        if !array.dictionary.is_null() || !schema.dictionary.is_null() {
            return Err(ImportError::UnexpectedDictionary);
        }
        if array.buffers.is_null() {
            return Err(ImportError::MissingBufferPointers);
        }

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
