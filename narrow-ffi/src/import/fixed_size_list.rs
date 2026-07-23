//! Borrowed import support for [`FixedSizeList`].

use core::{ffi::CStr, slice};

use narrow::{
    buffer::SliceBuffer,
    collection::flatten::Flatten,
    layout::{ArrayItem, fixed_size_list::FixedSizeList},
    length::Length,
    nullability::NonNullable,
};

use crate::{ArrowArray, ArrowSchema};

use super::{ImportError, ImportLayout};

impl<'array, T, const N: usize> ImportLayout<'array>
    for FixedSizeList<T, N, NonNullable, SliceBuffer<'array>>
where
    T: ArrayItem,
    T::Memory<SliceBuffer<'array>>: ImportLayout<'array>,
{
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

        // SAFETY: Common validation guarantees a one-entry child pointer array.
        let array_children = unsafe { slice::from_raw_parts(array.children, 1) };
        let child_array_pointer = array_children[0];
        if child_array_pointer.is_null() {
            return Err(ImportError::MissingArrayChildren);
        }
        // SAFETY: The caller guarantees that the child array is retained by
        // the parent for `'array`.
        let child_array: &'array ArrowArray = unsafe { &*child_array_pointer };

        // SAFETY: Common validation guarantees a one-entry child pointer array.
        let schema_children = unsafe { slice::from_raw_parts(schema.children, 1) };
        let child_schema_pointer = schema_children[0];
        if child_schema_pointer.is_null() {
            return Err(ImportError::MissingSchemaChildren);
        }
        // SAFETY: The caller guarantees that the child schema is valid while
        // the parent schema is borrowed.
        let child_schema = unsafe { &*child_schema_pointer };

        // SAFETY: The child structures are covered by the caller's Arrow C
        // Data guarantees and retained by their respective parents.
        let child = unsafe {
            <T::Memory<SliceBuffer<'array>> as ImportLayout>::import_layout(
                child_array,
                child_schema,
            )
        }?;
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

        Ok(Self::from_buffer(flattened))
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
        collection::{ChildRef, Collection, flatten::Flatten},
        layout::{fixed_size_list::FixedSizeList, fixed_size_primitive::FixedSizePrimitive},
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
