//! Export Narrow arrays through the Arrow C Data Interface.

extern crate alloc;

use alloc::boxed::Box;
use core::{
    borrow::Borrow,
    ffi::{CStr, c_void},
    ptr,
};

use narrow::{
    array::Array, buffer::Buffer, fixed_size::FixedSize,
    layout::fixed_size_primitive::FixedSizePrimitive, nullability::NonNullable,
};

use crate::{ArrowArray, ArrowSchema};

/// A fixed-size primitive with an Arrow C Data format string.
pub trait ArrowPrimitive: FixedSize {
    /// Arrow C Data type format.
    const FORMAT: &'static CStr;
}

macro_rules! primitive {
    ($($type:ty => $format:literal),+ $(,)?) => {
        $(
            impl ArrowPrimitive for $type {
                const FORMAT: &'static CStr = $format;
            }
        )+
    };
}

primitive! {
    i8 => c"c",
    u8 => c"C",
    i16 => c"s",
    u16 => c"S",
    i32 => c"i",
    u32 => c"I",
    i64 => c"l",
    u64 => c"L",
    f32 => c"f",
    f64 => c"g",
}

/// Export a Narrow array through the Arrow C Data Interface.
pub trait Export {
    /// Consumes the array and returns its Arrow C Data array and schema.
    fn export(self) -> (ArrowArray, ArrowSchema);
}

struct PrimitivePrivate<Values> {
    _values: Values,
    buffers: [*const c_void; 2],
}

impl<T, Storage> Export for Array<T, Storage>
where
    T: ArrowPrimitive,
    Storage: Buffer,
    Storage::For<T>: 'static,
{
    fn export(self) -> (ArrowArray, ArrowSchema) {
        let primitive: FixedSizePrimitive<T, NonNullable, Storage> = self.into_buffer();
        let values = primitive.into_buffer();
        let mut private = Box::new(PrimitivePrivate {
            _values: values,
            buffers: [ptr::null(); 2],
        });
        let values: &[T] = private._values.borrow();
        let length = i64::try_from(values.len()).expect("array length exceeds i64");
        private.buffers[1] = values.as_ptr().cast();

        let buffers = private.buffers.as_mut_ptr();
        let private_data = Box::into_raw(private).cast();
        let array = ArrowArray {
            length,
            null_count: 0,
            offset: 0,
            n_buffers: 2,
            n_children: 0,
            buffers,
            children: ptr::null_mut(),
            dictionary: ptr::null_mut(),
            release: Some(release_primitive::<Storage::For<T>>),
            private_data,
        };
        let schema = ArrowSchema {
            format: T::FORMAT.as_ptr(),
            name: c"".as_ptr(),
            metadata: ptr::null(),
            flags: 0,
            n_children: 0,
            children: ptr::null_mut(),
            dictionary: ptr::null_mut(),
            release: Some(release_schema),
            private_data: ptr::null_mut(),
        };
        (array, schema)
    }
}

unsafe extern "C" fn release_primitive<Values>(array: *mut ArrowArray) {
    // SAFETY: The Arrow C Data contract passes the live structure to its
    // producer-provided callback.
    let array = unsafe { &mut *array };
    let private_data = array.private_data;
    array.release = None;
    array.private_data = ptr::null_mut();
    array.buffers = ptr::null_mut();

    // SAFETY: `private_data` was created with `Box::into_raw` for this exact
    // `PrimitivePrivate<Values>` instantiation and is released only once.
    unsafe {
        drop(Box::from_raw(
            private_data.cast::<PrimitivePrivate<Values>>(),
        ))
    };
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
        buffer::{ArcBuffer, ArrayBuffer},
        layout::fixed_size_primitive::FixedSizePrimitive,
    };

    use super::{ArrowPrimitive, Export};

    #[test]
    fn primitive_format_strings_match_arrow() {
        assert_eq!(<i8 as ArrowPrimitive>::FORMAT, c"c");
        assert_eq!(<u8 as ArrowPrimitive>::FORMAT, c"C");
        assert_eq!(<i16 as ArrowPrimitive>::FORMAT, c"s");
        assert_eq!(<u16 as ArrowPrimitive>::FORMAT, c"S");
        assert_eq!(<i32 as ArrowPrimitive>::FORMAT, c"i");
        assert_eq!(<u32 as ArrowPrimitive>::FORMAT, c"I");
        assert_eq!(<i64 as ArrowPrimitive>::FORMAT, c"l");
        assert_eq!(<u64 as ArrowPrimitive>::FORMAT, c"L");
        assert_eq!(<f32 as ArrowPrimitive>::FORMAT, c"f");
        assert_eq!(<f64 as ArrowPrimitive>::FORMAT, c"g");
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
}
