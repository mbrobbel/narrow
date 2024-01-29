//! Array with fixed-size primitive values.

use super::Array;
use crate::{
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{Buffer, BufferType, VecBuffer},
    nullable::Nullable,
    validity::{Nullability, Validity},
    FixedSize, Index, Length,
};
use std::{ops, slice::SliceIndex};

/// Array with primitive values.
pub struct FixedSizePrimitiveArray<
    T: FixedSize,
    const NULLABLE: bool = false,
    Buffer: BufferType = VecBuffer,
>(pub <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>)
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>;

/// Generates type definitions for fixed-size primitive arrays with the known fixed-size types.
macro_rules! type_def {
    ($ident:ident, $ty:ty) => {
        #[doc = "Array with ["]
        #[doc = stringify!($ty)]
        #[doc = "] values."]
        pub type $ident<const NULLABLE: bool = false, Buffer = VecBuffer> =
            FixedSizePrimitiveArray<$ty, NULLABLE, Buffer>;
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

impl<T: FixedSize, const NULLABLE: bool, Buffer: BufferType>
    FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    for<'a> &'a <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>:
        IntoIterator,
{
    /// Returns an iterator over the items in this [`FixedSizePrimitiveArray`].
    pub fn iter(&self) -> <&'_ <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::IntoIter{
        <&Self as IntoIterator>::into_iter(self)
    }
}

impl<T: FixedSize, const NULLABLE: bool, Buffer: BufferType> Array
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    T: Nullability<NULLABLE>,
{
    type Item = <T as Nullability<NULLABLE>>::Item;
}

// todo(mbrobbel): buffer_ref traits?
impl<T: FixedSize, Buffer: BufferType> AsRef<[T]> for FixedSizePrimitiveArray<T, false, Buffer> {
    fn as_ref(&self) -> &[T] {
        self.0.as_slice()
    }
}

impl<T: FixedSize, const NULLABLE: bool, Buffer: BufferType> Default
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: FixedSize, U, const NULLABLE: bool, Buffer: BufferType> Extend<U>
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: FixedSize, Buffer: BufferType> From<FixedSizePrimitiveArray<T, false, Buffer>>
    for FixedSizePrimitiveArray<T, true, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Length,
    Bitmap<Buffer>: FromIterator<bool>,
{
    fn from(value: FixedSizePrimitiveArray<T, false, Buffer>) -> Self {
        Self(Nullable::from(value.0))
    }
}

impl<T: FixedSize, const NULLABLE: bool, U, Buffer: BufferType> FromIterator<U>
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: FixedSize, I: SliceIndex<[T]>, Buffer: BufferType> ops::Index<I>
    for FixedSizePrimitiveArray<T, false, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: ops::Index<I, Output = <I as SliceIndex<[T]>>::Output>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        ops::Index::index(&self.0, index)
    }
}

impl<T: FixedSize, Buffer: BufferType, const NULLABLE: bool> Index
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: Index,
{
    type Item<'a> = <<<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as Index>::Item<'a>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.0.index_unchecked(index)
    }
}

impl<'a, T: FixedSize, const NULLABLE: bool, Buffer: BufferType> IntoIterator
    for &'a FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    &'a <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: IntoIterator,
{
    type Item = <&'a <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::Item;
    type IntoIter = <&'a <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: FixedSize, const NULLABLE: bool, Buffer: BufferType> IntoIterator
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: IntoIterator,
{
    type Item = <<<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::Item;
    type IntoIter = <<<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: FixedSize, const NULLABLE: bool, Buffer: BufferType> Length
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: FixedSize, Buffer: BufferType> BitmapRef for FixedSizePrimitiveArray<T, true, Buffer> {
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<T: FixedSize, Buffer: BufferType> BitmapRefMut for FixedSizePrimitiveArray<T, true, Buffer> {
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<T: FixedSize, Buffer: BufferType> ValidityBitmap for FixedSizePrimitiveArray<T, true, Buffer> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bitmap::Bitmap,
        buffer::{Buffer, BufferRef, BufferRefMut},
    };
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
        let array = input.iter().collect::<FixedSizePrimitiveArray<_, true>>();
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
        let array = input.iter().collect::<FixedSizePrimitiveArray<_, true>>();
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
            .collect::<FixedSizePrimitiveArray<_, true>>();
        assert_eq!(array_nullable.len(), input_nullable.len());
    }

    #[test]
    fn convert_nullable() {
        let input = [1, 2, 3, 4];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        let mut nullable: FixedSizePrimitiveArray<_, true> = array.into();
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
            .collect::<FixedSizePrimitiveArray<_, true>>();
        assert_eq!(nullable.index_checked(1), None);
        assert_eq!(nullable.index_checked(3), Some(&4));
    }

    #[test]
    fn size_of() {
        assert_eq!(mem::size_of::<Int8Array>(), mem::size_of::<Vec<i8>>());
        assert_eq!(
            std::mem::size_of::<Int8Array<true>>(),
            mem::size_of::<Int8Array>() + mem::size_of::<Bitmap>()
        );
    }
}
