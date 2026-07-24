//! Borrowed import support for [`FixedSizeList`].

use core::ffi::CStr;

use narrow::{
    buffer::SliceBuffer,
    collection::flatten::Flatten,
    layout::{ArrayItem, fixed_size_list::FixedSizeList},
    length::Length,
};

use crate::{ArrowArray, ArrowSchema};

use super::{ImportError, ImportLayout, ImportNullability};

impl<'array, T, const N: usize, Nulls> ImportLayout<'array>
    for FixedSizeList<T, N, Nulls, SliceBuffer<'array>>
where
    T: ArrayItem,
    T::Memory<SliceBuffer<'array>>: ImportLayout<'array>,
    Nulls: ImportNullability<'array>,
{
    const FLAGS: i64 = Nulls::FLAGS;
    const BUFFERS: i64 = 1;
    const CHILDREN: i64 = 1;

    fn matches_format(format: &CStr) -> bool {
        let Some(digits) = format.to_bytes().strip_prefix(b"+w:") else {
            return false;
        };
        if digits.is_empty() {
            return false;
        }

        // Parse the decimal width without allocating, rejecting non-digits
        // and arithmetic overflow before comparing it with `N`.
        digits.iter().try_fold(0_usize, |size, byte| {
            let digit = (*byte).checked_sub(b'0')?;
            if digit > 9 {
                return None;
            }
            size.checked_mul(10)?.checked_add(usize::from(digit))
        }) == Some(N)
    }

    unsafe fn import_validated(
        array: &'array ArrowArray,
        schema: &ArrowSchema,
        length: usize,
    ) -> Result<Self, ImportError> {
        if N == 0 {
            return Err(ImportError::UnsupportedFixedSizeListSize { size: N });
        }

        // SAFETY: Common parent fields are validated and the caller upholds
        // the Arrow C Data requirements for the retained child structures.
        let child =
            unsafe { Self::import_child::<T::Memory<SliceBuffer<'array>>>(array, schema, 0) }?;
        let child_length = child.len();
        if length.checked_mul(N) != Some(child_length) {
            return Err(ImportError::FixedSizeListLengthMismatch {
                length,
                child_length,
                size: N,
            });
        }
        let flattened = Flatten::try_from_parts(child)
            .expect("validated fixed-size-list child length is a multiple of its width");

        // SAFETY: Common validation and the caller guarantee a valid optional
        // validity buffer for the imported lists.
        let collection = unsafe { Nulls::wrap(array, flattened) }?;
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
        collection::{ChildRef, Collection, flatten::Flatten},
        layout::{fixed_size_list::FixedSizeList, fixed_size_primitive::FixedSizePrimitive},
        validity::Validity,
    };

    use crate::{
        export::Export,
        import::{Import, ImportError},
    };

    #[test]
    fn imports_fixed_size_list_child_without_copying() {
        let storage = Arc::<[i32]>::from([1, 2, 3, 4]);
        let weak = Arc::downgrade(&storage);
        let data = storage.as_ptr();
        let values = FixedSizePrimitive::from_buffer(storage);
        let flattened = Flatten::try_from_parts(values).expect("valid fixed-size list");
        let source: Array<[i32; 2], ArcBuffer> =
            Array::from_buffer(FixedSizeList::from_buffer(flattened));
        let (array, schema) = source.export().expect("export array");

        {
            // SAFETY: The exported structures remain live and retain their
            // child array and value buffer for the imported array's lifetime.
            let imported: Array<[i32; 2], SliceBuffer<'_>> =
                unsafe { Import::import(&array, &schema) }.expect("import array");

            let imported_values: &[i32] = imported
                .buffer_ref()
                .buffer_ref()
                .child_ref()
                .buffer_ref()
                .borrow();
            assert_eq!(imported_values, [1, 2, 3, 4]);
            assert_eq!(imported_values.as_ptr(), data);
            assert_eq!(imported.owned(0), Some([1, 2]));
            assert_eq!(imported.owned(1), Some([3, 4]));
            assert!(weak.upgrade().is_some());
        };

        assert!(weak.upgrade().is_some());
        drop(array);
        assert!(weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn imports_nullable_fixed_size_list_without_copying() {
        let values = Arc::<[i32]>::from([1, 2, 3, 4]);
        let validity_values = Arc::<[u8]>::from([0b0000_0001]);
        let values_data = values.as_ptr();
        let validity_data = validity_values.as_ptr();
        let values_layout = FixedSizePrimitive::from_buffer(values);
        let flattened = Flatten::try_from_parts(values_layout).expect("valid fixed-size list");
        let bitmap =
            Bitmap::<ArcBuffer>::try_from_parts(validity_values, 2, 0).expect("valid bitmap");
        let validity = Validity::try_from_parts(flattened, bitmap).expect("valid parts");
        let source: Array<Option<[i32; 2]>, ArcBuffer> =
            Array::from_buffer(FixedSizeList::from_buffer(validity));
        let (mut array, schema) = source.export().expect("export array");
        array.null_count = -1;

        // SAFETY: The exported structures retain the child values and validity
        // buffer for the imported array's lifetime.
        let imported: Array<Option<[i32; 2]>, SliceBuffer<'_>> =
            unsafe { Import::import(&array, &schema) }.expect("import array");

        let imported_layout = imported.buffer_ref();
        let imported_validity = imported_layout.buffer_ref();
        let imported_bitmap = imported_validity.bitmap_ref().expect("validity bitmap");
        let imported_validity_values: &[u8] = imported_bitmap.buffer_ref().borrow();
        let imported_values: &[i32] = imported_validity
            .child_ref()
            .child_ref()
            .buffer_ref()
            .borrow();
        assert_eq!(imported.owned(0), Some(Some([1, 2])));
        assert_eq!(imported.owned(1), Some(None));
        assert_eq!(imported_validity_values.as_ptr(), validity_data);
        assert_eq!(imported_values.as_ptr(), values_data);
    }

    #[test]
    fn preserves_omitted_fixed_size_list_validity() {
        let source = [Some([1_i32, 2]), Some([3, 4])]
            .into_iter()
            .collect::<Array<Option<[i32; 2]>>>();
        let (mut array, schema) = source.export().expect("export array");
        // SAFETY: The exported array owns a one-entry buffer pointer array.
        unsafe { *array.buffers.cast_mut() = ptr::null() };
        array.null_count = 0;

        // SAFETY: The omitted validity buffer is permitted for an array with
        // no null items, and the exported child remains live.
        let imported: Array<Option<[i32; 2]>, SliceBuffer<'_>> =
            unsafe { Import::import(&array, &schema) }.expect("import array");

        assert!(imported.buffer_ref().buffer_ref().bitmap_ref().is_none());
        assert_eq!(imported.owned(0), Some(Some([1, 2])));
        assert_eq!(imported.owned(1), Some(Some([3, 4])));
    }

    #[test]
    fn rejects_mismatched_fixed_size_list_width() {
        let source = [[1_i32, 2]].into_iter().collect::<Array<[i32; 2]>>();
        let (array, mut schema) = source.export().expect("export array");
        schema.format = c"+w:3".as_ptr();

        // SAFETY: The exported structures and buffers remain valid; only the
        // parent format is changed to exercise width validation.
        let error = unsafe {
            <Array<[i32; 2], SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect_err("mismatched fixed-size-list width")
        };

        assert_eq!(error, ImportError::UnexpectedFormat);
    }

    #[test]
    fn rejects_mismatched_fixed_size_list_child_length() {
        let source = [[1_i32, 2], [3, 4]]
            .into_iter()
            .collect::<Array<[i32; 2]>>();
        let (mut array, schema) = source.export().expect("export array");
        array.length = 1;

        // SAFETY: The exported structures and buffers remain valid; only the
        // parent length is changed to exercise child length validation.
        let error = unsafe {
            <Array<[i32; 2], SliceBuffer<'_>> as Import>::import(&array, &schema)
                .expect_err("mismatched fixed-size-list child length")
        };

        assert_eq!(
            error,
            ImportError::FixedSizeListLengthMismatch {
                length: 1,
                child_length: 4,
                size: 2,
            }
        );
    }
}
