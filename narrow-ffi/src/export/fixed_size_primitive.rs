//! Export support for [`FixedSizePrimitive`].

use core::{borrow::Borrow, ffi::c_void, ptr};

use narrow::{
    bitmap::ValidityBitmap,
    buffer::{Buffer, BufferRef},
    collection::ChildRef,
    fixed_size::FixedSize,
    layout::fixed_size_primitive::FixedSizePrimitive,
    nullability::{NonNullable, Nullable},
};

use crate::{ArrowArray, ArrowSchema};

use super::{ArrowArrayLayout, ArrowType};

impl<T, Storage> ArrowArrayLayout for FixedSizePrimitive<T, NonNullable, Storage>
where
    T: FixedSize + ArrowType,
    Storage: Buffer,
{
    type Buffers = [*const c_void; 2];
    type Children = [ArrowArray; 0];

    fn schema() -> ArrowSchema {
        ArrowSchema::flat::<T>()
    }

    fn buffers(&self) -> Self::Buffers {
        let values: &[T] = self.buffer_ref().borrow();
        [ptr::null(), values.as_ptr().cast()]
    }
}

impl<T, Storage> ArrowArrayLayout for FixedSizePrimitive<T, Nullable, Storage>
where
    T: FixedSize + ArrowType,
    Storage: Buffer,
{
    type Buffers = [*const c_void; 2];
    type Children = [ArrowArray; 0];

    fn schema() -> ArrowSchema {
        ArrowSchema::flat::<Option<T>>()
    }

    fn null_count(&self) -> i64 {
        i64::try_from(ValidityBitmap::null_count(self.buffer_ref()))
            .expect("null count exceeds i64")
    }

    fn offset(&self) -> usize {
        self.buffer_ref()
            .bitmap_ref()
            .map_or(0, narrow::bitmap::Bitmap::bit_offset)
    }

    fn buffers(&self) -> Self::Buffers {
        let validity = self.buffer_ref();
        let validity_values = validity.bitmap_ref().map(|bitmap| {
            let values: &[u8] = bitmap.buffer_ref().borrow();
            values.as_ptr().cast()
        });
        let values: &[T] = validity.child_ref().borrow();
        [
            validity_values.unwrap_or(ptr::null()),
            values.as_ptr().cast(),
        ]
    }
}

impl ArrowType for i8 {
    const FORMAT: &'static core::ffi::CStr = c"c";
}

impl ArrowType for u8 {
    const FORMAT: &'static core::ffi::CStr = c"C";
}

impl ArrowType for i16 {
    const FORMAT: &'static core::ffi::CStr = c"s";
}

impl ArrowType for u16 {
    const FORMAT: &'static core::ffi::CStr = c"S";
}

impl ArrowType for i32 {
    const FORMAT: &'static core::ffi::CStr = c"i";
}

impl ArrowType for u32 {
    const FORMAT: &'static core::ffi::CStr = c"I";
}

impl ArrowType for i64 {
    const FORMAT: &'static core::ffi::CStr = c"l";
}

impl ArrowType for u64 {
    const FORMAT: &'static core::ffi::CStr = c"L";
}

impl ArrowType for f32 {
    const FORMAT: &'static core::ffi::CStr = c"f";
}

impl ArrowType for f64 {
    const FORMAT: &'static core::ffi::CStr = c"g";
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
        layout::fixed_size_primitive::FixedSizePrimitive,
        validity::Validity,
    };

    use crate::{ARROW_FLAG_NULLABLE, export::ExportError};

    use super::{super::Export, ArrowType};

    #[test]
    fn type_format_strings_match_arrow() {
        assert_eq!(i8::FORMAT, c"c");
        assert_eq!(u8::FORMAT, c"C");
        assert_eq!(i16::FORMAT, c"s");
        assert_eq!(u16::FORMAT, c"S");
        assert_eq!(i32::FORMAT, c"i");
        assert_eq!(u32::FORMAT, c"I");
        assert_eq!(i64::FORMAT, c"l");
        assert_eq!(u64::FORMAT, c"L");
        assert_eq!(f32::FORMAT, c"f");
        assert_eq!(f64::FORMAT, c"g");
        assert_eq!(<Option<i32>>::FORMAT, c"i");
        assert_eq!(<Option<i32>>::FLAGS, ARROW_FLAG_NULLABLE);
    }

    #[test]
    fn exports_primitive_array_without_copying_values() {
        let values = Arc::<[i32]>::from([1, 2, 3]);
        let weak = Arc::downgrade(&values);
        let data = values.as_ptr();
        let narrow_array: Array<i32, ArcBuffer> =
            Array::from_buffer(FixedSizePrimitive::from_buffer(values));

        let (array, schema) = narrow_array.export().expect("export array");

        assert_eq!(array.length, 3);
        assert_eq!(array.null_count, 0);
        assert_eq!(array.offset, 0);
        assert_eq!(array.n_buffers, 2);
        assert_eq!(array.n_children, 0);
        assert!(array.children.is_null());
        assert!(array.dictionary.is_null());
        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        assert!(buffers[0].is_null());
        assert_eq!(buffers[1], data.cast());
        assert_eq!(
            // SAFETY: The second buffer points to three i32 values held by the export.
            unsafe { slice::from_raw_parts(buffers[1].cast::<i32>(), 3) },
            [1, 2, 3]
        );

        // SAFETY: The exported schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(schema.format) }, c"i");
        assert_eq!(schema.flags, 0);
        assert_eq!(schema.n_children, 0);
        assert!(schema.children.is_null());
        assert!(schema.dictionary.is_null());
        assert!(weak.upgrade().is_some());

        drop(array);
        assert!(weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn pins_inline_values_for_export() {
        let narrow_array: Array<i32, ArrayBuffer<3>> =
            Array::from_buffer(FixedSizePrimitive::from_buffer([1, 2, 3]));

        let (array, _schema) = narrow_array.export().expect("export array");

        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        assert_eq!(
            // SAFETY: The second buffer points to three inline i32 values pinned in
            // the export's private allocation.
            unsafe { slice::from_raw_parts(buffers[1].cast::<i32>(), 3) },
            [1, 2, 3]
        );
    }

    #[test]
    fn exports_nullable_primitive_buffers_without_copying() {
        let values = Arc::<[i32]>::from([1, 0, 3]);
        let validity_values = Arc::<[u8]>::from([0b0000_0101]);
        let values_weak = Arc::downgrade(&values);
        let validity_weak = Arc::downgrade(&validity_values);
        let values_data = values.as_ptr();
        let validity_data = validity_values.as_ptr();
        let bitmap =
            Bitmap::<ArcBuffer>::try_from_parts(validity_values, 3, 0).expect("valid bitmap");
        let validity = Validity::try_from_parts(values, bitmap).expect("valid parts");
        let narrow_array: Array<Option<i32>, ArcBuffer> =
            Array::from_buffer(FixedSizePrimitive::from_buffer(validity));

        let (array, schema) = narrow_array.export().expect("export array");

        assert_eq!(array.length, 3);
        assert_eq!(array.null_count, 1);
        assert_eq!(array.offset, 0);
        assert_eq!(array.n_buffers, 2);
        assert_eq!(array.n_children, 0);
        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        assert_eq!(buffers[0], validity_data.cast());
        assert_eq!(buffers[1], values_data.cast());
        // SAFETY: The buffers point to storage retained by the export.
        assert_eq!(unsafe { *buffers[0].cast::<u8>() }, 0b0000_0101);
        assert_eq!(
            // SAFETY: The value buffer points to three i32 values.
            unsafe { slice::from_raw_parts(buffers[1].cast::<i32>(), 3) },
            [1, 0, 3]
        );

        // SAFETY: The exported schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(schema.format) }, c"i");
        assert_eq!(schema.flags, ARROW_FLAG_NULLABLE);
        assert!(values_weak.upgrade().is_some());
        assert!(validity_weak.upgrade().is_some());

        drop(array);
        assert!(values_weak.upgrade().is_none());
        assert!(validity_weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn exports_nullable_primitive_without_validity_bitmap() {
        let values = Arc::<[i32]>::from([1, 2, 3]);
        let weak = Arc::downgrade(&values);
        let values_data = values.as_ptr();
        let validity = Validity::<_, ArcBuffer>::from_collection(values);
        let narrow_array: Array<Option<i32>, ArcBuffer> =
            Array::from_buffer(FixedSizePrimitive::from_buffer(validity));

        let (array, schema) = narrow_array.export().expect("export array");

        assert_eq!(array.length, 3);
        assert_eq!(array.null_count, 0);
        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        assert!(buffers[0].is_null());
        assert_eq!(buffers[1], values_data.cast());
        assert_eq!(schema.flags, ARROW_FLAG_NULLABLE);
        assert!(weak.upgrade().is_some());

        drop(array);
        assert!(weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn rejects_non_zero_nullable_primitive_offset() {
        let bitmap = Bitmap::<ArrayBuffer<3>>::try_from_parts([0b0001_0100, 0, 0], 3, 2)
            .expect("valid bitmap");
        let validity = Validity::try_from_parts([1, 0, 3], bitmap).expect("valid parts");
        let array: Array<Option<i32>, ArrayBuffer<3>> =
            Array::from_buffer(FixedSizePrimitive::from_buffer(validity));

        let error = array.export().expect_err("non-zero offset");

        assert_eq!(error, ExportError::NonZeroOffset { offset: 2 });
    }
}
