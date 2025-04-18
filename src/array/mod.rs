//! Sequences of values with known length all having the same type.

use crate::{
    Length, NonNullable, Nullability, Nullable,
    buffer::{BufferType, VecBuffer},
    logical::{LogicalArray, LogicalArrayType},
    offset::{self, Offset},
};
use std::{collections::VecDeque, marker::PhantomData};

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
pub trait ArrayType<T: ?Sized> {
    /// The [`Array`] type for these items.
    ///
    /// It is generic over:
    /// - `Buffer`: a [`BufferType`] that is used for the array.
    /// - `OffsetItem`: an [`Offset`] that is used for arrays with offset
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
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType>: Array;
}

/// A helper type that allows extracting the [`ArrayType::Array`] type for any `ArrayType<T> for T`
pub type ArrayTypeOf<T, Buffer = VecBuffer, OffsetItem = offset::NA, UnionLayout = union::NA> =
    <T as ArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout>;

/// A helper type that allows extracting the [`ArrayType::Array`] type for any `ArrayType<Option<T>> for T`
pub type OptionArrayTypeOf<
    T,
    Buffer = VecBuffer,
    OffsetItem = offset::NA,
    UnionLayout = union::NA,
> = <Option<T> as ArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout>;

/// A helper type that allows extracting the [`ArrayType::Array`] type for any `ArrayType<T>::Item for <T as Nullability<NULLABLE>`
pub type NullableArrayTypeOf<
    Nullable,
    T,
    Buffer,
    OffsetItem = offset::NA,
    UnionLayout = union::NA,
> = <<Nullable as Nullability>::Item<T> as ArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout>;

impl<T: ArrayType<U> + ?Sized, U> ArrayType<U> for &T {
    type Array<Buffer: BufferType, OfsetItem: Offset, UnionLayout: UnionType> =
        <T as ArrayType<U>>::Array<Buffer, OfsetItem, UnionLayout>;
}

/// ?
pub trait Layout {
    /// a
    type Buffer: BufferType;
    /// b
    type Offset: Offset;
    /// c
    type Union: UnionType;
}

/// Implement [`ArrayType`] for `ty` using `array`.
macro_rules! impl_array_type {
    ($ty:ty, $array:ty) => {
        impl ArrayType<$ty> for $ty {
            type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> = $array;
        }
    };
    ($ty:ty, $array:ty, $inner:ty) => {
        impl ArrayType<$inner> for $ty {
            type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> = $array;
        }
    };
}

impl_array_type!(bool, BooleanArray<NonNullable, Buffer>);
impl_array_type!(Option<bool>, BooleanArray<Nullable, Buffer>, bool);

impl_array_type!(u8, FixedSizePrimitiveArray<u8, NonNullable, Buffer>);
impl_array_type!(Option<u8>, FixedSizePrimitiveArray<u8, Nullable, Buffer>, u8);
impl_array_type!(i8, FixedSizePrimitiveArray<i8, NonNullable, Buffer>);
impl_array_type!(Option<i8>, FixedSizePrimitiveArray<i8, Nullable, Buffer>, i8);
impl_array_type!(u16, FixedSizePrimitiveArray<u16, NonNullable, Buffer>);
impl_array_type!(Option<u16>, FixedSizePrimitiveArray<u16, Nullable, Buffer>, u16);
impl_array_type!(i16, FixedSizePrimitiveArray<i16, NonNullable, Buffer>);
impl_array_type!(Option<i16>, FixedSizePrimitiveArray<i16, Nullable, Buffer>, i16);
impl_array_type!(u32, FixedSizePrimitiveArray<u32, NonNullable, Buffer>);
impl_array_type!(Option<u32>, FixedSizePrimitiveArray<u32, Nullable, Buffer>, u32);
impl_array_type!(i32, FixedSizePrimitiveArray<i32, NonNullable, Buffer>);
impl_array_type!(Option<i32>, FixedSizePrimitiveArray<i32, Nullable, Buffer>, i32);
impl_array_type!(u64, FixedSizePrimitiveArray<u64, NonNullable, Buffer>);
impl_array_type!(Option<u64>, FixedSizePrimitiveArray<u64, Nullable, Buffer>, u64);
impl_array_type!(i64, FixedSizePrimitiveArray<i64, NonNullable, Buffer>);
impl_array_type!(Option<i64>, FixedSizePrimitiveArray<i64, Nullable, Buffer>, i64);
#[cfg(not(feature = "arrow-rs"))]
impl_array_type!(u128, FixedSizePrimitiveArray<u128, NonNullable, Buffer>);
#[cfg(not(feature = "arrow-rs"))]
impl_array_type!(Option<u128>, FixedSizePrimitiveArray<u128, Nullable, Buffer>, u128);
impl_array_type!(i128, FixedSizePrimitiveArray<i128, NonNullable, Buffer>);
impl_array_type!(Option<i128>, FixedSizePrimitiveArray<i128, Nullable, Buffer>, i128);

#[cfg(not(feature = "arrow-rs"))]
impl_array_type!(usize, FixedSizePrimitiveArray<usize, NonNullable, Buffer>);
#[cfg(not(feature = "arrow-rs"))]
impl_array_type!(Option<usize>, FixedSizePrimitiveArray<usize, Nullable, Buffer>, usize);
#[cfg(not(feature = "arrow-rs"))]
impl_array_type!(isize, FixedSizePrimitiveArray<isize, NonNullable, Buffer>);
#[cfg(not(feature = "arrow-rs"))]
impl_array_type!(Option<isize>, FixedSizePrimitiveArray<isize, Nullable, Buffer>, isize);

impl_array_type!(f32, FixedSizePrimitiveArray<f32, NonNullable, Buffer>);
impl_array_type!(Option<f32>, FixedSizePrimitiveArray<f32, Nullable, Buffer>, f32);
impl_array_type!(f64, FixedSizePrimitiveArray<f64, NonNullable, Buffer>);
impl_array_type!(Option<f64>, FixedSizePrimitiveArray<f64, Nullable, Buffer>, f64);

impl_array_type!((), NullArray<(), NonNullable, Buffer>);
impl_array_type!(Option<()>, NullArray<(), Nullable, Buffer>, ());

/// An byte array wrapper that maps to [`FixedSizeBinaryArray`] via its
/// [`ArrayType`] implementation. Used for example to map `Uuid` to
/// a [`FixedSizeBinaryArray`] instead of a [`FixedSizeListArray`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedSizeBinary<const N: usize>([u8; N]);

impl<const N: usize> Default for FixedSizeBinary<N> {
    fn default() -> Self {
        Self([u8::default(); N])
    }
}

impl<const N: usize> ArrayType<FixedSizeBinary<N>> for FixedSizeBinary<N> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        FixedSizeBinaryArray<N, NonNullable, Buffer>;
}

impl<const N: usize> ArrayType<FixedSizeBinary<N>> for Option<FixedSizeBinary<N>> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        FixedSizeBinaryArray<N, Nullable, Buffer>;
}

impl<const N: usize> From<&[u8; N]> for FixedSizeBinary<N> {
    fn from(value: &[u8; N]) -> Self {
        Self(*value)
    }
}

impl<const N: usize> From<[u8; N]> for FixedSizeBinary<N> {
    fn from(value: [u8; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize> From<FixedSizeBinary<N>> for [u8; N] {
    fn from(value: FixedSizeBinary<N>) -> Self {
        value.0
    }
}

/// An byte vector wrapper that maps to [`VariableSizeBinaryArray`] via its
/// [`ArrayType`] implementation. Used for example to map `Vec<u8>` to
/// a [`VariableSizeBinaryArray`] instead of a [`VariableSizeListArray`].
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariableSizeBinary(Vec<u8>);

impl ArrayType<VariableSizeBinary> for VariableSizeBinary {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<Self, NonNullable, Buffer, OffsetItem, UnionLayout>;
}

impl ArrayType<VariableSizeBinary> for Option<VariableSizeBinary> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<VariableSizeBinary, Nullable, Buffer, OffsetItem, UnionLayout>;
}

#[cfg(feature = "arrow-rs")]
impl crate::arrow::LogicalArrayType<VariableSizeBinary> for VariableSizeBinary {
    type ExtensionType = crate::arrow::NoExtensionType;
}

impl LogicalArrayType<VariableSizeBinary> for VariableSizeBinary {
    type ArrayType = Vec<u8>;

    fn from_array_type(item: Self::ArrayType) -> Self {
        item.into()
    }

    fn into_array_type(self) -> Self::ArrayType {
        self.into()
    }
}

impl From<Vec<u8>> for VariableSizeBinary {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<VariableSizeBinary> for Vec<u8> {
    fn from(value: VariableSizeBinary) -> Self {
        value.0
    }
}

impl Length for VariableSizeBinary {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl IntoIterator for VariableSizeBinary {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: ArrayType<T>, const N: usize> ArrayType<[T; N]> for [T; N] {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        FixedSizeListArray<N, ArrayTypeOf<T, Buffer, OffsetItem, UnionLayout>, NonNullable, Buffer>;
}
impl<T: ArrayType<T>, const N: usize> ArrayType<[T; N]> for Option<[T; N]> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        FixedSizeListArray<N, ArrayTypeOf<T, Buffer, OffsetItem, UnionLayout>, Nullable, Buffer>;
}
impl<T, const N: usize> ArrayType<[Option<T>; N]> for [Option<T>; N]
where
    Option<T>: ArrayType<T>,
{
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> = FixedSizeListArray<
        N,
        OptionArrayTypeOf<T, Buffer, OffsetItem, UnionLayout>,
        NonNullable,
        Buffer,
    >;
}
impl<T, const N: usize> ArrayType<[Option<T>; N]> for Option<[Option<T>; N]>
where
    Option<T>: ArrayType<T>,
{
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> = FixedSizeListArray<
        N,
        OptionArrayTypeOf<T, Buffer, OffsetItem, UnionLayout>,
        Nullable,
        Buffer,
    >;
}
impl ArrayType<str> for str {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        StringArray<NonNullable, OffsetItem, Buffer>;
}
impl<'a> ArrayType<&'a str> for &'a str {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        StringArray<NonNullable, OffsetItem, Buffer>;
}
impl<'a> ArrayType<&'a str> for Option<&'a str> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        StringArray<Nullable, OffsetItem, Buffer>;
}
impl ArrayType<String> for String {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        StringArray<NonNullable, OffsetItem, Buffer>;
}
impl ArrayType<String> for Option<String> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        StringArray<Nullable, OffsetItem, Buffer>;
}

impl<'a, T: ArrayType<T>> ArrayType<&'a [T]> for &'a [T] {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        VariableSizeListArray<ArrayTypeOf<T, Buffer>, NonNullable, OffsetItem, Buffer>;
}
impl<'a, T: ArrayType<T>> ArrayType<&'a [T]> for Option<&'a [T]> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        VariableSizeListArray<ArrayTypeOf<T, Buffer>, Nullable, OffsetItem, Buffer>;
}
impl<'a, T> ArrayType<&'a [Option<T>]> for &'a [Option<T>]
where
    Option<T>: ArrayType<T>,
{
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        VariableSizeListArray<OptionArrayTypeOf<T, Buffer>, NonNullable, OffsetItem, Buffer>;
}
impl<'a, T> ArrayType<&'a [Option<T>]> for Option<&'a [Option<T>]>
where
    Option<T>: ArrayType<T>,
{
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        VariableSizeListArray<OptionArrayTypeOf<T, Buffer>, Nullable, OffsetItem, Buffer>;
}
impl<T: ArrayType<T>> ArrayType<Vec<T>> for Vec<T> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        VariableSizeListArray<ArrayTypeOf<T, Buffer>, NonNullable, OffsetItem, Buffer>;
}
impl<T: ArrayType<T>> ArrayType<Vec<T>> for Option<Vec<T>> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        VariableSizeListArray<ArrayTypeOf<T, Buffer>, Nullable, OffsetItem, Buffer>;
}
impl<T> ArrayType<Vec<Option<T>>> for Vec<Option<T>>
where
    Option<T>: ArrayType<T>,
{
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        VariableSizeListArray<OptionArrayTypeOf<T, Buffer>, NonNullable, OffsetItem, Buffer>;
}
impl<T> ArrayType<Vec<Option<T>>> for Option<Vec<Option<T>>>
where
    Option<T>: ArrayType<T>,
{
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        VariableSizeListArray<OptionArrayTypeOf<T, Buffer>, Nullable, OffsetItem, Buffer>;
}

impl<T: ArrayType<T>> ArrayType<VecDeque<T>> for VecDeque<T> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        VariableSizeListArray<ArrayTypeOf<T, Buffer>, NonNullable, OffsetItem, Buffer>;
}
impl<T: ArrayType<T>> ArrayType<VecDeque<T>> for Option<VecDeque<T>> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        VariableSizeListArray<ArrayTypeOf<T, Buffer>, Nullable, OffsetItem, Buffer>;
}
impl<T> ArrayType<VecDeque<Option<T>>> for VecDeque<Option<T>>
where
    Option<T>: ArrayType<T>,
{
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        VariableSizeListArray<OptionArrayTypeOf<T, Buffer>, NonNullable, OffsetItem, Buffer>;
}
impl<T> ArrayType<VecDeque<Option<T>>> for Option<VecDeque<Option<T>>>
where
    Option<T>: ArrayType<T>,
{
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        VariableSizeListArray<OptionArrayTypeOf<T, Buffer>, Nullable, OffsetItem, Buffer>;
}

impl<T> ArrayType<PhantomData<T>> for PhantomData<T> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> = NullArray;
}
