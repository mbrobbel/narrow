//! Export support for [`VariableSizeList`].

extern crate alloc;

use alloc::boxed::Box;
use core::{borrow::Borrow, ffi::CStr, ffi::c_void, ptr};

use narrow::{
    buffer::{Buffer, BufferRef},
    collection::ChildRef,
    layout::{ArrayItem, variable_size_list::VariableSizeList},
    nullability::NonNullable,
    offset::Offset,
};

use crate::{ArrowArray, ArrowSchema};

use super::{ArrowArrayLayout, ExportError, release_schema};

/// An Arrow list offset with a C Data format string.
trait ArrowListOffset: Offset {
    /// Arrow C Data format for a list using this offset width.
    const FORMAT: &'static CStr;
}

impl ArrowListOffset for i32 {
    const FORMAT: &'static CStr = c"+l";
}

impl ArrowListOffset for i64 {
    const FORMAT: &'static CStr = c"+L";
}

impl<T, OffsetItem, Storage> ArrowArrayLayout
    for VariableSizeList<T, NonNullable, OffsetItem, Storage>
where
    T: ArrayItem,
    OffsetItem: ArrowListOffset,
    Storage: Buffer,
    T::Memory<Storage>: ArrowArrayLayout + 'static,
{
    type Buffers = [*const c_void; 2];
    type Children = [ArrowArray; 1];

    fn schema() -> ArrowSchema {
        ArrowSchema::variable_size_list::<OffsetItem>(<T::Memory<Storage>>::schema())
    }

    fn buffers(&self) -> Self::Buffers {
        let offsets: &[OffsetItem] = self.buffer_ref().buffer_ref().borrow();
        [ptr::null(), offsets.as_ptr().cast()]
    }

    fn children(&self) -> Result<Self::Children, ExportError> {
        Ok([self.buffer_ref().child_ref().child_array()?])
    }
}

impl ArrowSchema {
    /// Builds a variable-size-list schema and retains its child schema.
    fn variable_size_list<OffsetItem: ArrowListOffset>(child: Self) -> Self {
        let mut private = Box::new(VariableSizeListSchemaData {
            children: [child],
            child_pointers: [ptr::null_mut()],
        });
        private.child_pointers[0] = ptr::from_mut(&mut private.children[0]);

        let children = private.child_pointers.as_mut_ptr();
        let private_data = Box::into_raw(private).cast();
        Self {
            format: OffsetItem::FORMAT.as_ptr(),
            name: c"".as_ptr(),
            metadata: ptr::null(),
            flags: 0,
            n_children: 1,
            children,
            dictionary: ptr::null_mut(),
            release: Some(release_schema::<VariableSizeListSchemaData>),
            private_data,
        }
    }
}

/// Data retained by `ArrowSchema::private_data` for a variable-size list.
struct VariableSizeListSchemaData {
    /// Child schemas owned by the export.
    children: [ArrowSchema; 1],
    /// Arrow C Data child pointers.
    child_pointers: [*mut ArrowSchema; 1],
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::{sync::Arc, vec::Vec};
    use core::{ffi::CStr, slice};

    use narrow::{
        array::Array,
        buffer::ArcBuffer,
        layout::{fixed_size_primitive::FixedSizePrimitive, variable_size_list::VariableSizeList},
        offset::Offsets,
    };

    use super::{super::Export, ArrowListOffset};

    #[test]
    fn list_format_strings_match_arrow() {
        assert_eq!(<i32 as ArrowListOffset>::FORMAT, c"+l");
        assert_eq!(<i64 as ArrowListOffset>::FORMAT, c"+L");
    }

    #[test]
    fn exports_variable_size_list_without_copying_buffers() {
        let value_storage = Arc::<[i32]>::from([1, 2, 3]);
        let offset_storage = Arc::<[i32]>::from([0, 2, 3]);
        let values_weak = Arc::downgrade(&value_storage);
        let offsets_weak = Arc::downgrade(&offset_storage);
        let values_data = value_storage.as_ptr();
        let offsets_data = offset_storage.as_ptr();
        let values_layout = FixedSizePrimitive::from_buffer(value_storage);
        let offsets =
            Offsets::try_from_parts(values_layout, offset_storage).expect("valid offsets");
        let narrow_array: Array<Vec<i32>, ArcBuffer> =
            Array::from_buffer(VariableSizeList::from_buffer(offsets));

        let (array, schema) = narrow_array.export().expect("export array");

        assert_eq!(array.length, 2);
        assert_eq!(array.null_count, 0);
        assert_eq!(array.offset, 0);
        assert_eq!(array.n_buffers, 2);
        assert_eq!(array.n_children, 1);
        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        assert!(buffers[0].is_null());
        assert_eq!(buffers[1], offsets_data.cast());
        assert_eq!(
            // SAFETY: The offset buffer points to three values retained by the export.
            unsafe { slice::from_raw_parts(buffers[1].cast::<i32>(), 3) },
            [0, 2, 3]
        );
        // SAFETY: The exported array owns a one-entry child pointer array.
        let array_children = unsafe { slice::from_raw_parts(array.children, 1) };
        // SAFETY: The child pointer refers to an array retained by the parent.
        let child_array = unsafe { &*array_children[0] };
        assert_eq!(child_array.length, 3);
        // SAFETY: The child owns a two-entry buffer pointer array.
        let child_buffers = unsafe { slice::from_raw_parts(child_array.buffers, 2) };
        assert_eq!(child_buffers[1], values_data.cast());

        // SAFETY: The exported schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(schema.format) }, c"+l");
        assert_eq!(schema.flags, 0);
        assert_eq!(schema.n_children, 1);
        // SAFETY: The schema owns a one-entry child pointer array.
        let schema_children = unsafe { slice::from_raw_parts(schema.children, 1) };
        // SAFETY: The child pointer refers to a schema retained by the parent.
        let child_schema = unsafe { &*schema_children[0] };
        // SAFETY: The child schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(child_schema.format) }, c"i");
        assert!(values_weak.upgrade().is_some());
        assert!(offsets_weak.upgrade().is_some());

        drop(array);
        assert!(values_weak.upgrade().is_none());
        assert!(offsets_weak.upgrade().is_none());
        drop(schema);
    }
}
