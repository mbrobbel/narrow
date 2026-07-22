//! Export support for [`Boolean`].

use core::{borrow::Borrow, ffi::c_void, ptr};

use narrow::{
    bitmap::{BitmapRef, ValidityBitmap},
    buffer::{Buffer, BufferRef},
    collection::ChildRef,
    layout::boolean::Boolean,
    nullability::{NonNullable, Nullable},
};

use crate::{ArrowArray, ArrowSchema};

use super::{ArrowArrayLayout, ArrowType};

impl<Storage: Buffer> ArrowArrayLayout for Boolean<NonNullable, Storage> {
    type Buffers = [*const c_void; 2];
    type Children = [ArrowArray; 0];

    fn schema() -> ArrowSchema {
        ArrowSchema::flat::<bool>()
    }

    fn offset(&self) -> usize {
        self.buffer_ref().bit_offset()
    }

    fn buffers(&self) -> Self::Buffers {
        let values: &[u8] = self.buffer_ref().buffer_ref().borrow();
        [ptr::null(), values.as_ptr().cast()]
    }
}

impl<Storage: Buffer> ArrowArrayLayout for Boolean<Nullable, Storage> {
    type Buffers = [*const c_void; 2];
    type Children = [ArrowArray; 0];

    fn schema() -> ArrowSchema {
        ArrowSchema::flat::<Option<bool>>()
    }

    fn null_count(&self) -> i64 {
        i64::try_from(ValidityBitmap::null_count(self.buffer_ref()))
            .expect("null count exceeds i64")
    }

    fn offset(&self) -> usize {
        let validity = self.buffer_ref();
        // Surface either bitmap offset so the exporter rejects unsupported offsets.
        let validity_offset = validity.bitmap_ref().bit_offset();
        if validity_offset != 0 {
            return validity_offset;
        }
        validity.child_ref().bit_offset()
    }

    fn buffers(&self) -> Self::Buffers {
        let validity = self.buffer_ref();
        let validity_values: &[u8] = validity.bitmap_ref().buffer_ref().borrow();
        let values: &[u8] = validity.child_ref().buffer_ref().borrow();
        [validity_values.as_ptr().cast(), values.as_ptr().cast()]
    }
}

impl ArrowType for bool {
    const FORMAT: &'static core::ffi::CStr = c"b";
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
        layout::boolean::Boolean,
        validity::Validity,
    };

    use crate::{ARROW_FLAG_NULLABLE, export::ExportError};

    use super::{super::Export, ArrowType};

    #[test]
    fn type_format_string_matches_arrow() {
        assert_eq!(bool::FORMAT, c"b");
    }

    #[test]
    fn exports_boolean_bitmap_without_copying_values() {
        let values = Arc::<[u8]>::from([0b0000_0101]);
        let weak = Arc::downgrade(&values);
        let data = values.as_ptr();
        let bitmap = Bitmap::<ArcBuffer>::try_from_parts(values, 3, 0).expect("valid bitmap");
        let narrow_array: Array<bool, ArcBuffer> = Array::from_buffer(Boolean::from_buffer(bitmap));

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
        // SAFETY: The value buffer points to the byte held by the export.
        assert_eq!(unsafe { *buffers[1].cast::<u8>() }, 0b0000_0101);

        // SAFETY: The exported schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(schema.format) }, c"b");
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
    fn exports_nullable_boolean_bitmaps_without_copying() {
        let value_bytes = Arc::<[u8]>::from([0b0000_0001]);
        let validity_values = Arc::<[u8]>::from([0b0000_0101]);
        let values_weak = Arc::downgrade(&value_bytes);
        let validity_weak = Arc::downgrade(&validity_values);
        let values_data = value_bytes.as_ptr();
        let validity_data = validity_values.as_ptr();
        let value_bitmap =
            Bitmap::<ArcBuffer>::try_from_parts(value_bytes, 3, 0).expect("valid bitmap");
        let validity_bitmap =
            Bitmap::<ArcBuffer>::try_from_parts(validity_values, 3, 0).expect("valid bitmap");
        let validity =
            Validity::try_from_parts(value_bitmap, validity_bitmap).expect("valid parts");
        let narrow_array: Array<Option<bool>, ArcBuffer> =
            Array::from_buffer(Boolean::from_buffer(validity));

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
        // SAFETY: Both buffers point to bytes retained by the export.
        assert_eq!(unsafe { *buffers[0].cast::<u8>() }, 0b0000_0101);
        // SAFETY: The value buffer points to a byte retained by the export.
        assert_eq!(unsafe { *buffers[1].cast::<u8>() }, 0b0000_0001);

        // SAFETY: The exported schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(schema.format) }, c"b");
        assert_eq!(schema.flags, ARROW_FLAG_NULLABLE);
        assert!(values_weak.upgrade().is_some());
        assert!(validity_weak.upgrade().is_some());

        drop(array);
        assert!(values_weak.upgrade().is_none());
        assert!(validity_weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn rejects_non_zero_nullable_boolean_offsets() {
        let offset_values =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0001_0100], 3, 2).expect("valid bitmap");
        let zero_validity =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0000_0111], 3, 0).expect("valid bitmap");
        let offset_values_validity =
            Validity::try_from_parts(offset_values, zero_validity).expect("valid parts");
        let offset_values_array: Array<Option<bool>, ArrayBuffer<1>> =
            Array::from_buffer(Boolean::from_buffer(offset_values_validity));
        assert_eq!(
            offset_values_array
                .export()
                .expect_err("non-zero value offset"),
            ExportError::NonZeroOffset { offset: 2 }
        );

        let zero_values =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0000_0101], 3, 0).expect("valid bitmap");
        let offset_validity =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0001_1100], 3, 2).expect("valid bitmap");
        let offset_validity_values =
            Validity::try_from_parts(zero_values, offset_validity).expect("valid parts");
        let offset_validity_array: Array<Option<bool>, ArrayBuffer<1>> =
            Array::from_buffer(Boolean::from_buffer(offset_validity_values));
        assert_eq!(
            offset_validity_array
                .export()
                .expect_err("non-zero validity offset"),
            ExportError::NonZeroOffset { offset: 2 }
        );
    }

    #[test]
    fn pins_inline_boolean_bitmap_for_export() {
        let bitmap =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0000_0101], 3, 0).expect("valid bitmap");
        let narrow_array: Array<bool, ArrayBuffer<1>> =
            Array::from_buffer(Boolean::from_buffer(bitmap));

        let (array, _schema) = narrow_array.export().expect("export array");

        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        // SAFETY: The value buffer points to the inline byte pinned in the
        // export's private allocation.
        assert_eq!(unsafe { *buffers[1].cast::<u8>() }, 0b0000_0101);
    }

    #[test]
    fn rejects_non_zero_boolean_offset() {
        let bitmap =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0001_0100], 3, 2).expect("valid bitmap");
        let array: Array<bool, ArrayBuffer<1>> = Array::from_buffer(Boolean::from_buffer(bitmap));

        let error = array.export().expect_err("non-zero offset");

        assert_eq!(error, ExportError::NonZeroOffset { offset: 2 });
    }
}
