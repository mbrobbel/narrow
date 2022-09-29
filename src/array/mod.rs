//! Sequences of values with known length all having the same type.

use self::{
    boolean::BooleanArray,
    fixed_size_primitive::{
        Float32Array, Float64Array, Int16Array, Int32Array, Int64Array, Int8Array, Uint16Array,
        Uint32Array, Uint64Array, Uint8Array,
    },
    null::NullArray,
};
use crate::{buffer::Buffer, offset, Primitive};

pub mod boolean;
pub mod fixed_size_primitive;
pub mod null;
pub mod string;
pub mod r#struct;
pub mod variable_size_binary;

/// implemented by data structures that are arrow arrays
pub trait Array {
    type Item: ArrayType;
}

// todo(mb): variadic generics for buffer types (just generic for now)
pub trait ArrayType {
    /// The [Array] type that stores values of this type.
    type Array<
        DataBuffer: Buffer<Self::Primitive>,
        BitmapBuffer: Buffer<u8>,
        OffsetElement: offset::OffsetElement,
        OffsetBuffer: Buffer<OffsetElement>,
    >;

    /// Storage type in the data buffer. (This is weird for null arrays).
    type Primitive: Primitive;

    /// A reference type for this type that is used when borrowing data from the
    /// array.
    type RefItem<'a>;
}

macro_rules! impl_array_type {
    ($ty:ty, $prim:ty, $array:ty, $item:ty) => {
        impl ArrayType for $ty {
            type Array<
                DataBuffer: Buffer<Self::Primitive>,
                BitmapBuffer: Buffer<u8>,
                OffsetElement: offset::OffsetElement,
                OffsetBuffer: Buffer<OffsetElement>,
            > = $array;
            type Primitive = $prim;
            type RefItem<'a> = $item;
        }
    };
    ($ty:ty, $array:ty) => {
        impl_array_type!($ty, $ty, $array, $ty);
    };
}

impl_array_type!((), u8, NullArray<(), false, BitmapBuffer>, ());
impl_array_type!(Option<()>, u8, NullArray<(), true, BitmapBuffer>, Option<&'a()>);

impl_array_type!(bool, u8, BooleanArray<false, DataBuffer, BitmapBuffer>, bool);
impl_array_type!(Option<bool>, u8, BooleanArray<true, DataBuffer, BitmapBuffer>, Option<&'a bool>);

impl_array_type!(i8, Int8Array<false, DataBuffer, BitmapBuffer>);
impl_array_type!(Option<i8>, i8, Int8Array<true, DataBuffer, BitmapBuffer>, Option<&'a i8>);
impl_array_type!(i16, Int16Array<false, DataBuffer, BitmapBuffer>);
impl_array_type!(Option<i16>, i16, Int16Array<true, DataBuffer, BitmapBuffer>, Option<&'a i16>);
impl_array_type!(i32, Int32Array<false, DataBuffer, BitmapBuffer>);
impl_array_type!(Option<i32>, i32, Int32Array<true, DataBuffer, BitmapBuffer>, Option<&'a i32>);
impl_array_type!(i64, Int64Array<false, DataBuffer, BitmapBuffer>);
impl_array_type!(Option<i64>, i64, Int64Array<true, DataBuffer, BitmapBuffer>, Option<&'a i64>);

impl_array_type!(u8, Uint8Array<false, DataBuffer, BitmapBuffer>);
impl_array_type!(Option<u8>, u8, Uint8Array<true, DataBuffer, BitmapBuffer>, Option<&'a u8>);
impl_array_type!(u16, Uint16Array<false, DataBuffer, BitmapBuffer>);
impl_array_type!(Option<u16>, u16, Uint16Array<true, DataBuffer, BitmapBuffer>, Option<&'a u16>);
impl_array_type!(u32, Uint32Array<false, DataBuffer, BitmapBuffer>);
impl_array_type!(Option<u32>, u32, Uint32Array<true, DataBuffer, BitmapBuffer>, Option<&'a u32>);
impl_array_type!(u64, Uint64Array<false, DataBuffer, BitmapBuffer>);
impl_array_type!(Option<u64>, u64, Uint64Array<true, DataBuffer, BitmapBuffer>, Option<&'a u64>);

impl_array_type!(f32, Float32Array<false, DataBuffer, BitmapBuffer>);
impl_array_type!(Option<f32>, f32, Float32Array<true, DataBuffer, BitmapBuffer>, Option<&'a f32>);
impl_array_type!(f64, Float64Array<false, DataBuffer, BitmapBuffer>);
impl_array_type!(Option<f64>, f64, Float64Array<true, DataBuffer, BitmapBuffer>, Option<&'a f64>);
