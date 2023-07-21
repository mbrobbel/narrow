//! Sequences of values with known length all having the same type.

use crate::{buffer::BufferType, FixedSize};

mod null;
pub use null::*;

mod fixed_size_primitive;
pub use fixed_size_primitive::*;

mod r#struct;
pub use r#struct::*;

pub trait ArrayType {
    type Array<Buffer: BufferType>: Array;
}

pub trait Array {
    type Item: ArrayType;
}

macro_rules! impl_array_type {
    ($ty:ty, $array:ty) => {
        impl ArrayType for $ty {
            type Array<Buffer: BufferType> = $array;
        }
    };
}

impl_array_type!(u8, FixedSizePrimitiveArray<u8, false, Buffer>);
impl_array_type!(Option<u8>, FixedSizePrimitiveArray<u8, true, Buffer>);
impl_array_type!(i8, FixedSizePrimitiveArray<i8, false, Buffer>);
impl_array_type!(Option<i8>, FixedSizePrimitiveArray<i8, true, Buffer>);
impl_array_type!(u16, FixedSizePrimitiveArray<u16, false, Buffer>);
impl_array_type!(Option<u16>, FixedSizePrimitiveArray<u16, true, Buffer>);
impl_array_type!(i16, FixedSizePrimitiveArray<i16, false, Buffer>);
impl_array_type!(Option<i16>, FixedSizePrimitiveArray<i16, true, Buffer>);
impl_array_type!(u32, FixedSizePrimitiveArray<u32, false, Buffer>);
impl_array_type!(Option<u32>, FixedSizePrimitiveArray<u32, true, Buffer>);
impl_array_type!(i32, FixedSizePrimitiveArray<i32, false, Buffer>);
impl_array_type!(Option<i32>, FixedSizePrimitiveArray<i32, true, Buffer>);
impl_array_type!(u64, FixedSizePrimitiveArray<u64, false, Buffer>);
impl_array_type!(Option<u64>, FixedSizePrimitiveArray<u64, true, Buffer>);
impl_array_type!(i64, FixedSizePrimitiveArray<i64, false, Buffer>);
impl_array_type!(Option<i64>, FixedSizePrimitiveArray<i64, true, Buffer>);
impl_array_type!(u128, FixedSizePrimitiveArray<u128, false, Buffer>);
impl_array_type!(Option<u128>, FixedSizePrimitiveArray<u128, true, Buffer>);
impl_array_type!(i128, FixedSizePrimitiveArray<i128, false, Buffer>);
impl_array_type!(Option<i128>, FixedSizePrimitiveArray<i128, true, Buffer>);

impl_array_type!(usize, FixedSizePrimitiveArray<usize, false, Buffer>);
impl_array_type!(Option<usize>, FixedSizePrimitiveArray<usize, true, Buffer>);
impl_array_type!(isize, FixedSizePrimitiveArray<isize, false, Buffer>);
impl_array_type!(Option<isize>, FixedSizePrimitiveArray<isize, true, Buffer>);

impl_array_type!(f32, FixedSizePrimitiveArray<f32, false, Buffer>);
impl_array_type!(Option<f32>, FixedSizePrimitiveArray<f32, true, Buffer>);
impl_array_type!(f64, FixedSizePrimitiveArray<f64, false, Buffer>);
impl_array_type!(Option<f64>, FixedSizePrimitiveArray<f64, true, Buffer>);

impl_array_type!((), NullArray<(), false, Buffer>);
impl_array_type!(Option<()>, NullArray<(), true, Buffer>);

impl<T: FixedSize> ArrayType for (T,) {
    type Array<Buffer: BufferType> = FixedSizePrimitiveArray<(T,), false, Buffer>;
}
impl<T: FixedSize> ArrayType for Option<(T,)> {
    type Array<Buffer: BufferType> = FixedSizePrimitiveArray<(T,), true, Buffer>;
}

impl<T: FixedSize, const N: usize> ArrayType for [T; N] {
    type Array<Buffer: BufferType> = FixedSizePrimitiveArray<[T; N], false, Buffer>;
}
impl<T: FixedSize, const N: usize> ArrayType for Option<[T; N]> {
    type Array<Buffer: BufferType> = FixedSizePrimitiveArray<[T; N], true, Buffer>;
}
