//! Export support for [`FixedSizeList`].

extern crate alloc;

use alloc::{boxed::Box, ffi::CString, format};
use core::{ffi::c_void, ptr};

use narrow::{
    buffer::{Buffer, BufferRef},
    collection::ChildRef,
    layout::{ArrayItem, fixed_size_list::FixedSizeList},
    nullability::NonNullable,
};

use crate::{ArrowArray, ArrowSchema};

use super::{ArrowArrayLayout, ExportError, release_schema};

impl<T, const N: usize, Storage> ArrowArrayLayout for FixedSizeList<T, N, NonNullable, Storage>
where
    T: ArrayItem,
    Storage: Buffer,
    T::Memory<Storage>: ArrowArrayLayout + 'static,
{
    type Buffers = [*const c_void; 1];
    type Children = [ArrowArray; 1];

    fn schema() -> ArrowSchema {
        ArrowSchema::fixed_size_list::<N>(<T::Memory<Storage>>::schema())
    }

    fn buffers(&self) -> Self::Buffers {
        [ptr::null()]
    }

    fn children(&self) -> Result<Self::Children, ExportError> {
        Ok([self.buffer_ref().child_ref().child_array()?])
    }
}

impl ArrowSchema {
    /// Builds a fixed-size-list schema and retains its child schema.
    fn fixed_size_list<const N: usize>(child: Self) -> Self {
        let format_string = CString::new(format!("+w:{N}")).expect("valid fixed-size list format");
        let mut private = Box::new(FixedSizeListSchemaData {
            format: format_string,
            children: [child],
            child_pointers: [ptr::null_mut()],
        });
        private.child_pointers[0] = ptr::from_mut(&mut private.children[0]);

        let format_pointer = private.format.as_ptr();
        let children = private.child_pointers.as_mut_ptr();
        let private_data = Box::into_raw(private).cast();
        Self {
            format: format_pointer,
            name: c"".as_ptr(),
            metadata: ptr::null(),
            flags: 0,
            n_children: 1,
            children,
            dictionary: ptr::null_mut(),
            release: Some(release_schema::<FixedSizeListSchemaData>),
            private_data,
        }
    }
}

/// Data retained by `ArrowSchema::private_data` for a fixed-size list.
struct FixedSizeListSchemaData {
    /// Arrow C Data type format.
    format: CString,
    /// Child schemas owned by the export.
    children: [ArrowSchema; 1],
    /// Arrow C Data child pointers.
    child_pointers: [*mut ArrowSchema; 1],
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::sync::Arc;
    use core::{ffi::CStr, slice};

    use narrow::{
        array::Array,
        bitmap::Bitmap,
        buffer::{ArcBuffer, ArrayBuffer},
        collection::flatten::Flatten,
        layout::{fixed_size_list::FixedSizeList, fixed_size_primitive::FixedSizePrimitive},
        validity::Validity,
    };

    use super::super::{Export, ExportError};

    #[test]
    fn exports_fixed_size_list_child_without_copying_values() {
        let value_storage = Arc::<[i32]>::from([1, 2, 3, 4]);
        let weak = Arc::downgrade(&value_storage);
        let data = value_storage.as_ptr();
        let values_layout = FixedSizePrimitive::from_buffer(value_storage);
        let flattened = Flatten::try_from_parts(values_layout).expect("valid fixed-size list");
        let narrow_array: Array<[i32; 2], ArcBuffer> =
            Array::from_buffer(FixedSizeList::from_buffer(flattened));

        let (array, schema) = narrow_array.export().expect("export array");

        assert_eq!(array.length, 2);
        assert_eq!(array.null_count, 0);
        assert_eq!(array.offset, 0);
        assert_eq!(array.n_buffers, 1);
        assert_eq!(array.n_children, 1);
        // SAFETY: The exported array owns a one-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 1) };
        assert!(buffers[0].is_null());
        // SAFETY: The exported array owns a one-entry child pointer array.
        let array_children = unsafe { slice::from_raw_parts(array.children, 1) };
        // SAFETY: The child pointer refers to an array retained by the parent.
        let child_array = unsafe { &*array_children[0] };
        assert_eq!(child_array.length, 4);
        assert_eq!(child_array.n_buffers, 2);
        assert_eq!(child_array.n_children, 0);
        // SAFETY: The child owns a two-entry buffer pointer array.
        let child_buffers = unsafe { slice::from_raw_parts(child_array.buffers, 2) };
        assert!(child_buffers[0].is_null());
        assert_eq!(child_buffers[1], data.cast());
        assert_eq!(
            // SAFETY: The value buffer points to four values retained by the parent.
            unsafe { slice::from_raw_parts(child_buffers[1].cast::<i32>(), 4) },
            [1, 2, 3, 4]
        );

        // SAFETY: The exported schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(schema.format) }, c"+w:2");
        assert_eq!(schema.flags, 0);
        assert_eq!(schema.n_children, 1);
        // SAFETY: The schema owns a one-entry child pointer array.
        let schema_children = unsafe { slice::from_raw_parts(schema.children, 1) };
        // SAFETY: The child pointer refers to a schema retained by the parent.
        let child_schema = unsafe { &*schema_children[0] };
        // SAFETY: The child schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(child_schema.format) }, c"i");
        assert_eq!(child_schema.flags, 0);
        assert_eq!(child_schema.n_children, 0);
        assert!(weak.upgrade().is_some());

        drop(array);
        assert!(weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn rejects_non_zero_fixed_size_list_child_offset() {
        let bitmap = Bitmap::<ArrayBuffer<4>>::try_from_parts([0b0011_1100, 0, 0, 0], 4, 2)
            .expect("valid bitmap");
        let validity = Validity::try_from_parts([1, 2, 3, 4], bitmap).expect("valid parts");
        let values_layout = FixedSizePrimitive::from_buffer(validity);
        let flattened = Flatten::try_from_parts(values_layout).expect("valid fixed-size list");
        let array: Array<[Option<i32>; 2], ArrayBuffer<4>> =
            Array::from_buffer(FixedSizeList::from_buffer(flattened));

        let error = array.export().expect_err("non-zero child offset");

        assert_eq!(error, ExportError::NonZeroOffset { offset: 2 });
    }
}
