//! Export support for [`VariableSizeList`].

extern crate alloc;

use alloc::{boxed::Box, sync::Arc};
use core::ptr;

use narrow::{
    bitmap::ValidityBitmap,
    buffer::{Buffer, BufferRef},
    collection::ChildRef,
    layout::{ArrayItem, variable_size_list::VariableSizeList},
    nullability::{NonNullable, Nullability, Nullable},
};

use crate::{ARROW_FLAG_NULLABLE, ArrowListOffset, ArrowSchema};

use super::{ArrowArrayLayout, BufferPointer, Checker, ExportError, Exported, release_schema};

impl<T, OffsetItem, Storage> ArrowArrayLayout
    for VariableSizeList<T, NonNullable, OffsetItem, Storage>
where
    T: ArrayItem,
    OffsetItem: ArrowListOffset,
    Storage: Buffer,
    T::Memory<Storage>: ArrowArrayLayout + 'static,
{
    type Buffers = [BufferPointer; 2];
    type Children = [Exported; 1];

    fn schema() -> ArrowSchema {
        ArrowSchema::variable_size_list::<OffsetItem, NonNullable>(<T::Memory<Storage>>::schema())
    }

    fn buffers(&self, checker: &mut Checker<'_>) -> Result<Self::Buffers, ExportError> {
        Ok([
            checker.all_valid(),
            checker.offsets(self.buffer_ref().buffer_ref())?,
        ])
    }

    fn children<Root: Send + Sync + 'static>(
        &self,
        root: &Arc<Root>,
    ) -> Result<Self::Children, ExportError> {
        Ok([self.buffer_ref().child_ref().array(root)?])
    }
}

impl<T, OffsetItem, Storage> ArrowArrayLayout for VariableSizeList<T, Nullable, OffsetItem, Storage>
where
    T: ArrayItem,
    OffsetItem: ArrowListOffset,
    Storage: Buffer,
    T::Memory<Storage>: ArrowArrayLayout + 'static,
{
    type Buffers = [BufferPointer; 2];
    type Children = [Exported; 1];

    fn schema() -> ArrowSchema {
        ArrowSchema::variable_size_list::<OffsetItem, Nullable>(<T::Memory<Storage>>::schema())
    }

    fn buffers(&self, checker: &mut Checker<'_>) -> Result<Self::Buffers, ExportError> {
        let validity = checker.validity(self.buffer_ref().bitmap_ref())?;
        Ok([
            validity,
            checker.offsets(self.buffer_ref().child_ref().buffer_ref())?,
        ])
    }

    fn children<Root: Send + Sync + 'static>(
        &self,
        root: &Arc<Root>,
    ) -> Result<Self::Children, ExportError> {
        Ok([self.buffer_ref().child_ref().child_ref().array(root)?])
    }
}

impl ArrowSchema {
    /// Builds a variable-size-list schema and retains its child schema.
    fn variable_size_list<OffsetItem: ArrowListOffset, Nulls: Nullability>(child: Self) -> Self {
        // Leak the box before deriving pointers into it: moving it afterwards
        // would invalidate them.
        let private = Box::into_raw(Box::new(VariableSizeListSchemaData {
            children: [child],
            child_pointers: [ptr::null_mut()],
        }));
        // SAFETY: `private` comes from `Box::into_raw` and nothing else
        // references it until `release_schema` reclaims it.
        let data = unsafe { &mut *private };
        data.child_pointers[0] = ptr::from_mut(&mut data.children[0]);

        let children = data.child_pointers.as_mut_ptr();
        let private_data = private.cast();
        Self {
            format: OffsetItem::FORMAT.as_ptr(),
            name: c"".as_ptr(),
            metadata: ptr::null(),
            flags: if Nulls::NULLABLE {
                ARROW_FLAG_NULLABLE
            } else {
                0
            },
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
        bitmap::Bitmap,
        buffer::{ArcBuffer, ArrayBuffer},
        layout::{fixed_size_primitive::FixedSizePrimitive, variable_size_list::VariableSizeList},
        offset::Offsets,
        validity::Validity,
    };

    use crate::{ARROW_FLAG_NULLABLE, ArrowListOffset};

    use super::super::{Export, ExportError, move_first_child};

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

    #[test]
    fn exports_nullable_variable_size_list_without_copying_buffers() {
        let value_storage = Arc::<[i32]>::from([1, 2, 3]);
        let offset_storage = Arc::<[i32]>::from([0, 2, 3]);
        let validity_storage = Arc::<[u8]>::from([0b0000_0001]);
        let values_weak = Arc::downgrade(&value_storage);
        let offsets_weak = Arc::downgrade(&offset_storage);
        let validity_weak = Arc::downgrade(&validity_storage);
        let values_data = value_storage.as_ptr();
        let offsets_data = offset_storage.as_ptr();
        let validity_data = validity_storage.as_ptr();
        let values_layout = FixedSizePrimitive::from_buffer(value_storage);
        let offsets =
            Offsets::try_from_parts(values_layout, offset_storage).expect("valid offsets");
        let bitmap =
            Bitmap::<ArcBuffer>::try_from_parts(validity_storage, 2, 0).expect("valid bitmap");
        let validity = Validity::try_from_parts(offsets, bitmap).expect("valid parts");
        let narrow_array: Array<Option<Vec<i32>>, ArcBuffer> =
            Array::from_buffer(VariableSizeList::from_buffer(validity));

        let (array, schema) = narrow_array.export().expect("export array");

        assert_eq!(array.length, 2);
        assert_eq!(array.null_count, 1);
        assert_eq!(array.offset, 0);
        assert_eq!(array.n_buffers, 2);
        assert_eq!(array.n_children, 1);
        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        assert_eq!(buffers[0], validity_data.cast());
        assert_eq!(buffers[1], offsets_data.cast());
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
        assert_eq!(schema.flags, ARROW_FLAG_NULLABLE);
        // SAFETY: The schema owns a one-entry child pointer array.
        let schema_children = unsafe { slice::from_raw_parts(schema.children, 1) };
        // SAFETY: The child pointer refers to a schema retained by the parent.
        let child_schema = unsafe { &*schema_children[0] };
        assert_eq!(child_schema.flags, 0);
        assert!(values_weak.upgrade().is_some());
        assert!(offsets_weak.upgrade().is_some());
        assert!(validity_weak.upgrade().is_some());

        drop(array);
        assert!(values_weak.upgrade().is_none());
        assert!(offsets_weak.upgrade().is_none());
        assert!(validity_weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn moved_child_outlives_parent() {
        let value_storage = Arc::<[i32]>::from([1, 2, 3]);
        let values_weak = Arc::downgrade(&value_storage);
        let values_layout = FixedSizePrimitive::from_buffer(value_storage);
        let offsets = Offsets::try_from_parts(values_layout, Arc::<[i32]>::from([0, 2, 3]))
            .expect("valid offsets");
        let narrow_array: Array<Vec<i32>, ArcBuffer> =
            Array::from_buffer(VariableSizeList::from_buffer(offsets));
        let (array, _schema) = narrow_array.export().expect("export array");

        let child = move_first_child(&array);
        drop(array);

        assert!(values_weak.upgrade().is_some());
        // SAFETY: The child owns a two-entry buffer pointer array.
        let child_buffers = unsafe { slice::from_raw_parts(child.buffers, 2) };
        assert_eq!(
            // SAFETY: The value buffer points to three values retained by the child.
            unsafe { slice::from_raw_parts(child_buffers[1].cast::<i32>(), 3) },
            [1, 2, 3]
        );
        drop(child);
        assert!(values_weak.upgrade().is_none());
    }

    #[test]
    fn moved_child_with_inline_storage_outlives_parent() {
        // The values live inline in the exported layout, so reading them
        // after it is freed is only detected by Miri.
        let values_layout = FixedSizePrimitive::from_buffer([1, 2, 3]);
        let offsets = Offsets::try_from_parts(values_layout, [0, 2, 3]).expect("valid offsets");
        let narrow_array: Array<Vec<i32>, ArrayBuffer<3>> =
            Array::from_buffer(VariableSizeList::from_buffer(offsets));
        let (array, _schema) = narrow_array.export().expect("export array");

        let child = move_first_child(&array);
        drop(array);

        // SAFETY: The child owns a two-entry buffer pointer array.
        let child_buffers = unsafe { slice::from_raw_parts(child.buffers, 2) };
        assert_eq!(
            // SAFETY: The value buffer points to three values retained by the child.
            unsafe { slice::from_raw_parts(child_buffers[1].cast::<i32>(), 3) },
            [1, 2, 3]
        );
    }

    #[test]
    fn moved_grandchild_outlives_ancestors() {
        let value_storage = Arc::<[i32]>::from([1, 2, 3]);
        let values_weak = Arc::downgrade(&value_storage);
        let values_layout = FixedSizePrimitive::from_buffer(value_storage);
        let inner = Offsets::try_from_parts(values_layout, Arc::<[i32]>::from([0, 2, 3]))
            .expect("valid offsets");
        let outer = Offsets::try_from_parts(
            VariableSizeList::from_buffer(inner),
            Arc::<[i32]>::from([0, 1, 2]),
        )
        .expect("valid offsets");
        let narrow_array: Array<Vec<Vec<i32>>, ArcBuffer> =
            Array::from_buffer(VariableSizeList::from_buffer(outer));
        let (array, _schema) = narrow_array.export().expect("export array");

        let child = move_first_child(&array);
        drop(array);
        let grandchild = move_first_child(&child);
        drop(child);

        assert!(values_weak.upgrade().is_some());
        // SAFETY: The grandchild owns a two-entry buffer pointer array.
        let grandchild_buffers = unsafe { slice::from_raw_parts(grandchild.buffers, 2) };
        assert_eq!(
            // SAFETY: The value buffer points to three values retained by the grandchild.
            unsafe { slice::from_raw_parts(grandchild_buffers[1].cast::<i32>(), 3) },
            [1, 2, 3]
        );
        drop(grandchild);
        assert!(values_weak.upgrade().is_none());
    }

    #[test]
    fn rejects_non_zero_nullable_variable_size_list_offset() {
        let values_layout = FixedSizePrimitive::from_buffer([1, 2, 3]);
        let offsets = Offsets::try_from_parts(values_layout, [0, 2, 3]).expect("valid offsets");
        let bitmap = Bitmap::<ArrayBuffer<3>>::try_from_parts([0b0000_1100, 0, 0], 2, 2)
            .expect("valid bitmap");
        let validity = Validity::try_from_parts(offsets, bitmap).expect("valid parts");
        let array: Array<Option<Vec<i32>>, ArrayBuffer<3>> =
            Array::from_buffer(VariableSizeList::from_buffer(validity));

        let error = array.export().expect_err("non-zero offset");

        assert_eq!(error, ExportError::NonZeroOffset { offset: 2 });
    }
}
