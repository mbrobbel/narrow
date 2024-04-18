//! Sequences of values with known length all having the same type.

use crate::{
    buffer::BufferType,
    offset::{self, OffsetElement},
};
use std::collections::VecDeque;

mod boolean;
pub use boolean::*;

mod fixed_size_binary;
pub use fixed_size_binary::*;

mod fixed_size_list;
pub use fixed_size_list::*;

mod fixed_size_primitive;
pub use fixed_size_primitive::*;

mod null;
pub use null::*;

mod string;
pub use string::*;

mod r#struct;
pub use r#struct::*;

pub mod union;
pub use union::*;

mod variable_size_binary;
pub use variable_size_binary::*;

mod variable_size_list;
pub use variable_size_list::*;

/// Types that store their data in Arrow arrays.
pub trait Array {
    /// The items stored in this array.
    type Item;
}

/// Types that can be stored in Arrow arrays.
// Note: the generic `T` is required to allow impls on foreign wrappers e.g.
// Option. (https://rust-lang.github.io/rfcs/2451-re-rebalancing-coherence.html)
pub trait ArrayType<T: ?Sized = Self> {
    /// The [`Array`] type for these items.
    ///
    /// It is generic over:
    /// - `Buffer`: a [`BufferType`] that is used for the array.
    /// - `OffsetItem`: an [`OffsetElement`] that is used for arrays with offset
    ///   buffers.
    /// - `UnionLayout`: a [`UnionType`] that is used for union arrays.
    ///
    /// When using this type constructor for arrays that have no offset buffer
    /// [`offset::NA`] should be used to indicate that this does not apply.
    ///
    /// When using this type constructor to construct arrays that are not union
    /// arrays [`union::NA`] should be used to indicate that does not apply.
    ///
    /// This still ends up setting the default type, but this is needed because
    /// there are no default types for generic associated types.
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>: Array;
}

impl<T: ArrayType + ?Sized> ArrayType for &T {
    type Array<Buffer: BufferType, OfsetItem: OffsetElement, UnionLayout: UnionType> =
        <T as ArrayType>::Array<Buffer, OfsetItem, UnionLayout>;
}

/// Implement [`ArrayType`] for `ty` using `array`.
macro_rules! impl_array_type {
    ($ty:ty, $array:ty) => {
        impl ArrayType for $ty {
            type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
                $array;
        }
    };
}

impl_array_type!(bool, BooleanArray<false, Buffer>);
impl_array_type!(Option<bool>, BooleanArray<true, Buffer>);

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
#[cfg(not(feature = "arrow-rs"))]
impl_array_type!(u128, FixedSizePrimitiveArray<u128, false, Buffer>);
#[cfg(not(feature = "arrow-rs"))]
impl_array_type!(Option<u128>, FixedSizePrimitiveArray<u128, true, Buffer>);
impl_array_type!(i128, FixedSizePrimitiveArray<i128, false, Buffer>);
impl_array_type!(Option<i128>, FixedSizePrimitiveArray<i128, true, Buffer>);

#[cfg(not(feature = "arrow-rs"))]
impl_array_type!(usize, FixedSizePrimitiveArray<usize, false, Buffer>);
#[cfg(not(feature = "arrow-rs"))]
impl_array_type!(Option<usize>, FixedSizePrimitiveArray<usize, true, Buffer>);
#[cfg(not(feature = "arrow-rs"))]
impl_array_type!(isize, FixedSizePrimitiveArray<isize, false, Buffer>);
#[cfg(not(feature = "arrow-rs"))]
impl_array_type!(Option<isize>, FixedSizePrimitiveArray<isize, true, Buffer>);

impl_array_type!(f32, FixedSizePrimitiveArray<f32, false, Buffer>);
impl_array_type!(Option<f32>, FixedSizePrimitiveArray<f32, true, Buffer>);
impl_array_type!(f64, FixedSizePrimitiveArray<f64, false, Buffer>);
impl_array_type!(Option<f64>, FixedSizePrimitiveArray<f64, true, Buffer>);

impl_array_type!((), NullArray<(), false, Buffer>);
impl_array_type!(Option<()>, NullArray<(), true, Buffer>);

impl<T: ArrayType, const N: usize> ArrayType for [T; N] {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        FixedSizeListArray<
            N,
            <T as ArrayType>::Array<Buffer, OffsetItem, UnionLayout>,
            false,
            Buffer,
        >;
}
impl<T: ArrayType, const N: usize> ArrayType for Option<[T; N]> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        FixedSizeListArray<
            N,
            <T as ArrayType>::Array<Buffer, OffsetItem, UnionLayout>,
            true,
            Buffer,
        >;
}

impl ArrayType for str {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        StringArray<false, OffsetItem, Buffer>;
}
impl<'a> ArrayType for Option<&'a str> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        StringArray<true, OffsetItem, Buffer>;
}
impl ArrayType for String {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        StringArray<false, OffsetItem, Buffer>;
}
impl ArrayType for Option<String> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        StringArray<true, OffsetItem, Buffer>;
}

impl<'a, T: ArrayType> ArrayType for &'a [T] {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        VariableSizeListArray<
            <T as ArrayType>::Array<Buffer, offset::NA, union::NA>,
            false,
            OffsetItem,
            Buffer,
        >;
}
impl<'a, T: ArrayType> ArrayType for Option<&'a [T]> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        VariableSizeListArray<
            <T as ArrayType>::Array<Buffer, offset::NA, union::NA>,
            true,
            OffsetItem,
            Buffer,
        >;
}
impl<T: ArrayType> ArrayType for Vec<T> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        VariableSizeListArray<
            <T as ArrayType>::Array<Buffer, offset::NA, union::NA>,
            false,
            OffsetItem,
            Buffer,
        >;
}
impl<T: ArrayType> ArrayType for Option<Vec<T>> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        VariableSizeListArray<
            <T as ArrayType>::Array<Buffer, offset::NA, union::NA>,
            true,
            OffsetItem,
            Buffer,
        >;
}
impl<T: ArrayType> ArrayType for VecDeque<T> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        VariableSizeListArray<
            <T as ArrayType>::Array<Buffer, offset::NA, union::NA>,
            false,
            OffsetItem,
            Buffer,
        >;
}
impl<T: ArrayType> ArrayType for Option<VecDeque<T>> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        VariableSizeListArray<
            <T as ArrayType>::Array<Buffer, offset::NA, union::NA>,
            true,
            OffsetItem,
            Buffer,
        >;
}
