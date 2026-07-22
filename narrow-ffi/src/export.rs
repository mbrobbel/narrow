//! Export [`Array`] values through the Arrow C Data Interface.

extern crate alloc;

use alloc::boxed::Box;
use core::{
    borrow::Borrow,
    ffi::{CStr, c_void},
    ptr,
};

use narrow::{
    array::Array,
    bitmap::Bitmap,
    buffer::Buffer,
    fixed_size::FixedSize,
    layout::{ArrayItem, boolean::Boolean, fixed_size_primitive::FixedSizePrimitive},
    nullability::NonNullable,
};

use crate::{ArrowArray, ArrowSchema};

/// A type with an [Arrow C Data format string].
///
/// [Arrow C Data format string]: https://arrow.apache.org/docs/format/CDataInterface.html#data-type-description-format-strings
pub trait ArrowType {
    /// Arrow C Data type format.
    const FORMAT: &'static CStr;
}

impl ArrowType for bool {
    const FORMAT: &'static CStr = c"b";
}

impl ArrowType for i8 {
    const FORMAT: &'static CStr = c"c";
}

impl ArrowType for u8 {
    const FORMAT: &'static CStr = c"C";
}

impl ArrowType for i16 {
    const FORMAT: &'static CStr = c"s";
}

impl ArrowType for u16 {
    const FORMAT: &'static CStr = c"S";
}

impl ArrowType for i32 {
    const FORMAT: &'static CStr = c"i";
}

impl ArrowType for u32 {
    const FORMAT: &'static CStr = c"I";
}

impl ArrowType for i64 {
    const FORMAT: &'static CStr = c"l";
}

impl ArrowType for u64 {
    const FORMAT: &'static CStr = c"L";
}

impl ArrowType for f32 {
    const FORMAT: &'static CStr = c"f";
}

impl ArrowType for f64 {
    const FORMAT: &'static CStr = c"g";
}

/// Export an [`Array`] through the Arrow C Data Interface.
pub trait Export {
    /// Consumes `self` and returns an [`ArrowArray`] and [`ArrowSchema`].
    fn export(self) -> (ArrowArray, ArrowSchema);
}

/// Data retained by `ArrowArray::private_data` for an array export.
struct ArrayData<Values> {
    /// Storage backing the exported value buffer.
    values: Values,
    /// Arrow C Data buffer pointers for validity and values.
    buffers: [*const c_void; 2],
}

/// A flat, non-nullable layout backed by one value buffer.
trait FlatArrayLayout {
    /// Item stored in the value buffer.
    type Value: FixedSize;
    /// Owned value-buffer storage retained by the export.
    type Values: Borrow<[Self::Value]> + 'static;

    /// Returns the value storage, logical length, and logical offset.
    fn into_parts(self) -> (Self::Values, usize, usize);
}

impl<T, Storage> FlatArrayLayout for FixedSizePrimitive<T, NonNullable, Storage>
where
    T: FixedSize + ArrowType,
    Storage: Buffer,
    Storage::For<T>: 'static,
{
    type Value = T;
    type Values = Storage::For<T>;

    fn into_parts(self) -> (Self::Values, usize, usize) {
        let values = self.into_buffer();
        let length = values.borrow().len();
        (values, length, 0)
    }
}

impl<Storage> FlatArrayLayout for Boolean<NonNullable, Storage>
where
    Storage: Buffer,
    Storage::For<u8>: 'static,
{
    type Value = u8;
    type Values = Storage::For<u8>;

    fn into_parts(self) -> (Self::Values, usize, usize) {
        let bitmap: Bitmap<Storage> = self.into_buffer();
        bitmap.into_parts()
    }
}

impl<T, Storage> Export for Array<T, Storage>
where
    T: ArrayItem + ArrowType,
    Storage: Buffer,
    T::Memory<Storage>: FlatArrayLayout,
{
    fn export(self) -> (ArrowArray, ArrowSchema) {
        // Remove the layout wrappers without copying their storage.
        let (values, length, offset) = self.into_buffer().into_parts();

        // Pin the storage before exposing its address so inline buffers remain
        // at a stable location for the lifetime of the export.
        let mut private = Box::new(ArrayData {
            values,
            buffers: [ptr::null(); 2],
        });
        let length = i64::try_from(length).expect("array length exceeds i64");
        let offset = i64::try_from(offset).expect("array offset exceeds i64");

        // Flat arrays have validity and value buffers. A non-nullable array may
        // expose a null validity buffer because its null count is zero.
        private.buffers[1] = private.values.borrow().as_ptr().cast();

        // Keep the value storage and its buffer pointer array alive until the
        // consumer invokes the release callback.
        let buffers = private.buffers.as_mut_ptr();
        let private_data = Box::into_raw(private).cast();
        let array = ArrowArray {
            length,
            null_count: 0,
            offset,
            n_buffers: 2,
            n_children: 0,
            buffers,
            children: ptr::null_mut(),
            dictionary: ptr::null_mut(),
            release: Some(
                release_array::<ArrayData<<T::Memory<Storage> as FlatArrayLayout>::Values>>,
            ),
            private_data,
        };

        (array, ArrowSchema::flat::<T>())
    }
}

impl ArrowSchema {
    fn flat<T: ArrowType>() -> Self {
        Self {
            format: T::FORMAT.as_ptr(),
            name: c"".as_ptr(),
            metadata: ptr::null(),
            flags: 0,
            n_children: 0,
            children: ptr::null_mut(),
            dictionary: ptr::null_mut(),
            release: Some(release_schema),
            private_data: ptr::null_mut(),
        }
    }
}

unsafe extern "C" fn release_array<PrivateData>(array: *mut ArrowArray) {
    // SAFETY: The Arrow C Data contract passes the live structure to its
    // producer-provided callback.
    let array = unsafe { &mut *array };
    let private_data = array.private_data;
    array.release = None;
    array.private_data = ptr::null_mut();
    array.buffers = ptr::null_mut();

    // SAFETY: `private_data` was created with `Box::into_raw` for this exact
    // `PrivateData` type and is released only once.
    unsafe { drop(Box::from_raw(private_data.cast::<PrivateData>())) };
}

unsafe extern "C" fn release_schema(schema: *mut ArrowSchema) {
    // SAFETY: The Arrow C Data contract passes the live structure to its
    // producer-provided callback.
    unsafe { (*schema).release = None };
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
        layout::{boolean::Boolean, fixed_size_primitive::FixedSizePrimitive},
    };

    use super::{ArrowType, Export};

    #[test]
    fn type_format_strings_match_arrow() {
        assert_eq!(bool::FORMAT, c"b");
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
    }

    #[test]
    fn exports_primitive_array_without_copying_values() {
        let values = Arc::<[i32]>::from([1, 2, 3]);
        let weak = Arc::downgrade(&values);
        let data = values.as_ptr();
        let array: Array<i32, ArcBuffer> =
            Array::from_buffer(FixedSizePrimitive::from_buffer(values));

        let (array, schema) = array.export();

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
        // SAFETY: The second buffer points to three i32 values held by the export.
        assert_eq!(
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
        let array: Array<i32, ArrayBuffer<3>> =
            Array::from_buffer(FixedSizePrimitive::from_buffer([1, 2, 3]));

        let (array, _schema) = array.export();

        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        // SAFETY: The second buffer points to three inline i32 values pinned in
        // the export's private allocation.
        assert_eq!(
            unsafe { slice::from_raw_parts(buffers[1].cast::<i32>(), 3) },
            [1, 2, 3]
        );
    }

    #[test]
    fn exports_boolean_bitmap_without_copying_values() {
        let values = Arc::<[u8]>::from([0b0001_0100]);
        let weak = Arc::downgrade(&values);
        let data = values.as_ptr();
        let bitmap = Bitmap::<ArcBuffer>::try_from_parts(values, 3, 2).expect("valid bitmap");
        let array: Array<bool, ArcBuffer> = Array::from_buffer(Boolean::from_buffer(bitmap));

        let (array, schema) = array.export();

        assert_eq!(array.length, 3);
        assert_eq!(array.null_count, 0);
        assert_eq!(array.offset, 2);
        assert_eq!(array.n_buffers, 2);
        assert_eq!(array.n_children, 0);
        assert!(array.children.is_null());
        assert!(array.dictionary.is_null());
        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        assert!(buffers[0].is_null());
        assert_eq!(buffers[1], data.cast());
        // SAFETY: The value buffer points to the byte held by the export.
        assert_eq!(unsafe { *buffers[1].cast::<u8>() }, 0b0001_0100);

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
    fn pins_inline_boolean_bitmap_for_export() {
        let bitmap =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0000_0101], 3, 0).expect("valid bitmap");
        let array: Array<bool, ArrayBuffer<1>> = Array::from_buffer(Boolean::from_buffer(bitmap));

        let (array, _schema) = array.export();

        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        // SAFETY: The value buffer points to the inline byte pinned in the
        // export's private allocation.
        assert_eq!(unsafe { *buffers[1].cast::<u8>() }, 0b0000_0101);
    }
}
