//! Array with fixed-size primitive values.

use super::Array;
use crate::{
    FixedSize, Index, Length,
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{Buffer, BufferType, VecBuffer},
    nullability::{NonNullable, Nullability, Nullable},
    validity::Validity,
};
use std::{
    fmt::{Debug, Formatter, Result},
    ops,
    slice::SliceIndex,
};

/// Array with primitive values.
pub struct FixedSizePrimitiveArray<
    T: FixedSize,
    Nullable: Nullability = NonNullable,
    Buffer: BufferType = VecBuffer,
>(pub Nullable::Collection<Buffer::Buffer<T>, Buffer>);

/// Generates type definitions for fixed-size primitive arrays with the known fixed-size types.
macro_rules! type_def {
    ($ident:ident, $ty:ty) => {
        #[doc = "Array with ["]
        #[doc = stringify!($ty)]
        #[doc = "] values."]
        pub type $ident<Nullable = NonNullable, Buffer = VecBuffer> =
            FixedSizePrimitiveArray<$ty, Nullable, Buffer>;
    };
}

type_def!(Int8Array, i8);
type_def!(Int16Array, i16);
type_def!(Int32Array, i32);
type_def!(Int64Array, i64);
#[cfg(not(feature = "arrow-rs"))]
type_def!(Int128Array, i128);
type_def!(Uint8Array, u8);
type_def!(Uint16Array, u16);
type_def!(Uint32Array, u32);
type_def!(Uint64Array, u64);
#[cfg(not(feature = "arrow-rs"))]
type_def!(Uint128Array, u128);

type_def!(IsizeArray, isize);
type_def!(UsizeArray, usize);

type_def!(Float32Array, f32);
type_def!(Float64Array, f64);

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType>
    FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    for<'a> &'a Nullable::Collection<Buffer::Buffer<T>, Buffer>: IntoIterator,
{
    /// Returns an iterator over the items in this [`FixedSizePrimitiveArray`].
    pub fn iter(
        &self,
    ) -> <&'_ Nullable::Collection<Buffer::Buffer<T>, Buffer> as IntoIterator>::IntoIter {
        <&Self as IntoIterator>::into_iter(self)
    }
}

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> Array
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
{
    type Item = Nullable::Item<T>;
}

// todo(mbrobbel): buffer_ref traits?
impl<T: FixedSize, Buffer: BufferType> AsRef<[T]>
    for FixedSizePrimitiveArray<T, NonNullable, Buffer>
{
    fn as_ref(&self) -> &[T] {
        self.0.as_slice()
    }
}

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> Clone
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    Nullable::Collection<Buffer::Buffer<T>, Buffer>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> Debug
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    Nullable::Collection<Buffer::Buffer<T>, Buffer>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_tuple("FixedSizePrimitiveArray")
            .field(&self.0)
            .finish()
    }
}

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> Default
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    Nullable::Collection<Buffer::Buffer<T>, Buffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: FixedSize, U, Nullable: Nullability, Buffer: BufferType> Extend<U>
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    Nullable::Collection<Buffer::Buffer<T>, Buffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: FixedSize, Buffer: BufferType> From<FixedSizePrimitiveArray<T, NonNullable, Buffer>>
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    Buffer::Buffer<T>: Length,
    Bitmap<Buffer>: FromIterator<bool>,
{
    fn from(value: FixedSizePrimitiveArray<T, NonNullable, Buffer>) -> Self {
        Self(Validity::from(value.0))
    }
}

impl<T: FixedSize, Nullable: Nullability, U, Buffer: BufferType> FromIterator<U>
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    Nullable::Collection<Buffer::Buffer<T>, Buffer>: FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: FixedSize, I: SliceIndex<[T]>, Buffer: BufferType> ops::Index<I>
    for FixedSizePrimitiveArray<T, NonNullable, Buffer>
where
    Buffer::Buffer<T>: ops::Index<I, Output = <I as SliceIndex<[T]>>::Output>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        ops::Index::index(&self.0, index)
    }
}

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> Index
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    Nullable::Collection<Buffer::Buffer<T>, Buffer>: Index,
{
    type Item<'a>
        = <Nullable::Collection<Buffer::Buffer<T>, Buffer> as Index>::Item<'a>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.0.index_unchecked(index)
    }
}

impl<'a, T: FixedSize, Nullable: Nullability, Buffer: BufferType> IntoIterator
    for &'a FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    &'a Nullable::Collection<Buffer::Buffer<T>, Buffer>: IntoIterator,
{
    type Item = <&'a Nullable::Collection<Buffer::Buffer<T>, Buffer> as IntoIterator>::Item;
    type IntoIter = <&'a Nullable::Collection<Buffer::Buffer<T>, Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> IntoIterator
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    Nullable::Collection<Buffer::Buffer<T>, Buffer>: IntoIterator,
{
    type Item = <Nullable::Collection<Buffer::Buffer<T>, Buffer> as IntoIterator>::Item;
    type IntoIter = <Nullable::Collection<Buffer::Buffer<T>, Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> Length
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    Nullable::Collection<Buffer::Buffer<T>, Buffer>: Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: FixedSize, Buffer: BufferType> BitmapRef for FixedSizePrimitiveArray<T, Nullable, Buffer> {
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<T: FixedSize, Buffer: BufferType> BitmapRefMut
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
{
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<T: FixedSize, Buffer: BufferType> PartialEq
    for FixedSizePrimitiveArray<T, NonNullable, Buffer>
{
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl<T: FixedSize, Buffer: BufferType> PartialEq<[T]>
    for FixedSizePrimitiveArray<T, NonNullable, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: PartialEq<[T]>,
{
    fn eq(&self, other: &[T]) -> bool {
        self.0.eq(other)
    }
}

impl<T: FixedSize, Buffer: BufferType> PartialEq<[Option<T>]>
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    for<'a> &'a Self: IntoIterator<Item = Option<T>>,
{
    fn eq(&self, other: &[Option<T>]) -> bool {
        self.len() == other.len() && self.into_iter().zip(other).all(|(a, &b)| a == b)
    }
}

impl<T: FixedSize, Buffer: BufferType> ValidityBitmap
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{BufferRef, BufferRefMut};
    use std::mem;

    #[test]
    fn from_iter() {
        let input = [1_u8, 2, 3, 4];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.0.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(array.0.as_slice(), array.0.as_bytes());

        #[cfg(not(feature = "arrow-rs"))]
        {
            let input_array = [[1_u8, 2], [3, 4]];
            let array_array = input_array
                .into_iter()
                .collect::<FixedSizePrimitiveArray<_>>();
            assert_eq!(array_array.0.as_slice(), &[[1, 2], [3, 4]]);
            assert_eq!(<_ as Buffer<u8>>::as_bytes(&array_array.0), &[1, 2, 3, 4]);
        };
    }

    #[test]
    fn from_iter_nullable() {
        let input = [Some(1_u64), None, Some(3), Some(4)];
        let array = input
            .iter()
            .collect::<FixedSizePrimitiveArray<_, Nullable>>();
        assert_eq!(array.0.buffer_ref().as_slice(), &[1, u64::default(), 3, 4]);
        assert_eq!(array.is_valid(0), Some(true));
        assert_eq!(array.is_null(1), Some(true));
        assert_eq!(array.is_valid(2), Some(true));
        assert_eq!(array.is_valid(3), Some(true));
        assert_eq!(array.is_valid(4), None);
    }

    #[test]
    fn into_iter() {
        let input = [1_u8, 2, 3, 4];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), input);

        #[cfg(not(feature = "arrow-rs"))]
        {
            let input_array = [[1_u8, 2], [3, 4]];
            let array_array = input_array
                .into_iter()
                .collect::<FixedSizePrimitiveArray<_>>();
            assert_eq!(array_array.into_iter().collect::<Vec<_>>(), input_array);
        };
    }

    #[test]
    fn into_iter_nullable() {
        let input = [Some(1_u64), None, Some(3), Some(4)];
        let array = input
            .iter()
            .collect::<FixedSizePrimitiveArray<_, Nullable>>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), input);
    }

    #[test]
    fn length() {
        let input = [1_u8, 2, 3, 4];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.len(), input.as_slice().len());

        #[cfg(not(feature = "arrow-rs"))]
        {
            let input_array = [[1_u8, 2], [3, 4]];
            let array_array = input_array
                .into_iter()
                .collect::<FixedSizePrimitiveArray<_>>();
            assert_eq!(array_array.len(), input_array.as_slice().len());
        };

        let input_nullable = [Some(1_u64), None, Some(3), Some(4)];
        let array_nullable = input_nullable
            .iter()
            .collect::<FixedSizePrimitiveArray<_, Nullable>>();
        assert_eq!(array_nullable.len(), input_nullable.len());
    }

    #[test]
    fn convert_nullable() {
        let input = [1, 2, 3, 4];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        let mut nullable: FixedSizePrimitiveArray<_, Nullable> = array.into();
        nullable.bitmap_ref_mut().buffer_ref_mut().as_mut_slice()[0] = 0b0000_1101;
        assert_eq!(
            nullable.into_iter().collect::<Vec<_>>(),
            [Some(1), None, Some(3), Some(4)]
        );
    }

    #[test]
    fn index() {
        let array = [1, 2, 3, 4]
            .into_iter()
            .collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array[..3], [1, 2, 3]);
        assert_eq!(array[3], 4);
        assert_eq!(array.index_checked(3), &4);

        let nullable = [Some(1), None, Some(3), Some(4)]
            .into_iter()
            .collect::<FixedSizePrimitiveArray<_, Nullable>>();
        assert_eq!(nullable.index_checked(1), None);
        assert_eq!(nullable.index_checked(3), Some(&4));
    }

    #[test]
    fn size_of() {
        assert_eq!(mem::size_of::<Int8Array>(), mem::size_of::<Vec<i8>>());
        assert_eq!(
            std::mem::size_of::<Int8Array<Nullable>>(),
            mem::size_of::<Int8Array>() + mem::size_of::<Bitmap>()
        );
    }
}
