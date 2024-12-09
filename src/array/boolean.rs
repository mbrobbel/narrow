//! Array with boolean values.

use super::Array;
use crate::{
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{BufferRef, BufferRefMut, BufferType, VecBuffer},
    nullability::{NonNullable, Nullability, Nullable},
    validity::Validity,
    Index, Length,
};
use std::fmt::{Debug, Formatter, Result};

/// Array with boolean values.
///
/// Values are stored using single bits in a [Bitmap].
pub struct BooleanArray<Nullable: Nullability = NonNullable, Buffer: BufferType = VecBuffer>(
    pub(crate) Nullable::Collection<Bitmap<Buffer>, Buffer>,
);

impl<Nullable: Nullability, Buffer: BufferType> BooleanArray<Nullable, Buffer>
where
    for<'a> &'a Nullable::Collection<Bitmap<Buffer>, Buffer>: IntoIterator,
{
    /// Returns an iterator over the boolean items in this [`BooleanArray`].
    pub fn iter(
        &self,
    ) -> <&'_ Nullable::Collection<Bitmap<Buffer>, Buffer> as IntoIterator>::IntoIter {
        <&Self as IntoIterator>::into_iter(self)
    }
}

impl<Nullable: Nullability, Buffer: BufferType> Array for BooleanArray<Nullable, Buffer> {
    type Item = Nullable::Item<bool>;
}

impl<Nullable: Nullability, Buffer: BufferType> BufferRef<u8> for BooleanArray<Nullable, Buffer>
where
    Nullable::Collection<Bitmap<Buffer>, Buffer>: BufferRef<u8>,
{
    type Buffer = <Nullable::Collection<Bitmap<Buffer>, Buffer> as BufferRef<u8>>::Buffer;

    fn buffer_ref(&self) -> &Self::Buffer {
        self.0.buffer_ref()
    }
}

impl<Nullable: Nullability, Buffer: BufferType> BufferRefMut<u8> for BooleanArray<Nullable, Buffer>
where
    Nullable::Collection<Bitmap<Buffer>, Buffer>: BufferRefMut<u8>,
{
    type BufferMut = <Nullable::Collection<Bitmap<Buffer>, Buffer> as BufferRefMut<u8>>::BufferMut;

    fn buffer_ref_mut(&mut self) -> &mut Self::BufferMut {
        self.0.buffer_ref_mut()
    }
}

impl<Nullable: Nullability, Buffer: BufferType> Clone for BooleanArray<Nullable, Buffer>
where
    Nullable::Collection<Bitmap<Buffer>, Buffer>: Clone,
{
    fn clone(&self) -> Self {
        BooleanArray(self.0.clone())
    }
}

impl<Nullable: Nullability, Buffer: BufferType> Debug for BooleanArray<Nullable, Buffer>
where
    Nullable::Collection<Bitmap<Buffer>, Buffer>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_tuple("BooleanArray").field(&self.0).finish()
    }
}

impl<Nullable: Nullability, Buffer: BufferType> Default for BooleanArray<Nullable, Buffer>
where
    Nullable::Collection<Bitmap<Buffer>, Buffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<U, Nullable: Nullability, Buffer: BufferType> Extend<U> for BooleanArray<Nullable, Buffer>
where
    Nullable::Collection<Bitmap<Buffer>, Buffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<Buffer: BufferType> From<BooleanArray<NonNullable, Buffer>> for BooleanArray<Nullable, Buffer>
where
    Bitmap<Buffer>: FromIterator<bool>,
{
    fn from(value: BooleanArray<NonNullable, Buffer>) -> Self {
        Self(Validity::from(value.0))
    }
}

impl<Nullable: Nullability, U, Buffer: BufferType> FromIterator<U>
    for BooleanArray<Nullable, Buffer>
where
    Nullable::Collection<Bitmap<Buffer>, Buffer>: FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<Nullable: Nullability, Buffer: BufferType> Index for BooleanArray<Nullable, Buffer>
where
    Nullable::Collection<Bitmap<Buffer>, Buffer>: Index,
{
    type Item<'a>
        = <Nullable::Collection<Bitmap<Buffer>, Buffer> as Index>::Item<'a>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.0.index_unchecked(index)
    }
}

impl<'a, Nullable: Nullability, Buffer: BufferType> IntoIterator
    for &'a BooleanArray<Nullable, Buffer>
where
    &'a Nullable::Collection<Bitmap<Buffer>, Buffer>: IntoIterator,
{
    type Item = <&'a Nullable::Collection<Bitmap<Buffer>, Buffer> as IntoIterator>::Item;
    type IntoIter = <&'a Nullable::Collection<Bitmap<Buffer>, Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<Nullable: Nullability, Buffer: BufferType> IntoIterator for BooleanArray<Nullable, Buffer>
where
    Nullable::Collection<Bitmap<Buffer>, Buffer>: IntoIterator,
{
    type Item = <Nullable::Collection<Bitmap<Buffer>, Buffer> as IntoIterator>::Item;
    type IntoIter = <Nullable::Collection<Bitmap<Buffer>, Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<Nullable: Nullability, Buffer: BufferType> Length for BooleanArray<Nullable, Buffer>
where
    Nullable::Collection<Bitmap<Buffer>, Buffer>: Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<Buffer: BufferType> BitmapRef for BooleanArray<Nullable, Buffer> {
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<Buffer: BufferType> BitmapRefMut for BooleanArray<Nullable, Buffer> {
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<Buffer: BufferType> ValidityBitmap for BooleanArray<Nullable, Buffer> {}

impl<Buffer: BufferType> PartialEq<[bool]> for BooleanArray<NonNullable, Buffer>
where
    Bitmap<Buffer>: PartialEq<[bool]>,
{
    fn eq(&self, other: &[bool]) -> bool {
        self.0.eq(other)
    }
}

impl<Buffer: BufferType> PartialEq<[Option<bool>]> for BooleanArray<Nullable, Buffer>
where
    for<'a> &'a Self: IntoIterator<Item = Option<bool>>,
{
    fn eq(&self, other: &[Option<bool>]) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(a, &b)| a == b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::BoxBuffer;
    use std::mem;

    #[test]
    fn from_iter() {
        let mut array = [true, false, true, true]
            .into_iter()
            .collect::<BooleanArray<NonNullable, BoxBuffer>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.buffer_ref().as_ref(), [0b0000_1101]);
        array.buffer_ref_mut()[0] = 0xff;
        assert_eq!(array.buffer_ref().as_ref(), [0b1111_1111]);
    }

    #[test]
    fn from_iter_nullable() {
        let array = [Some(true), None, Some(true), Some(false)]
            .into_iter()
            .collect::<BooleanArray<Nullable>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0.data.is_valid(0), Some(true));
        assert_eq!(array.0.data.is_null(1), Some(true));
        assert_eq!(array.0.data.is_valid(2), Some(true));
        assert_eq!(array.0.data.is_valid(3), Some(false));
        assert_eq!(array.is_valid(0), Some(true));
        assert_eq!(array.is_null(1), Some(true));
        assert_eq!(array.is_valid(2), Some(true));
        assert_eq!(array.is_valid(3), Some(true));
        assert_eq!(array.bitmap_ref().get(0), Some(true));
        assert_eq!(array.bitmap_ref().get(1), Some(false));
        assert_eq!(array.bitmap_ref().get(2), Some(true));
        assert_eq!(array.bitmap_ref().get(3), Some(true));
        assert!(array.0.data.is_valid(4).is_none());
        assert_eq!(array.0.data.bitmap_ref().len(), array.len());
    }

    #[test]
    fn into_iter() {
        let input = [true, false, true, true];
        let array = input.iter().collect::<BooleanArray>();
        assert_eq!(input, (&array).into_iter().collect::<Vec<_>>().as_slice());

        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());
    }

    #[test]
    fn into_iter_nullable() {
        let input = [Some(true), None, Some(true), Some(false)];
        let array = input.into_iter().collect::<BooleanArray<Nullable>>();
        assert_eq!(input, (&array).into_iter().collect::<Vec<_>>().as_slice());

        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());
    }

    #[test]
    fn index() {
        let array = [true, false, true, true].iter().collect::<BooleanArray>();
        assert_eq!(array.index(0), Some(true));

        let nullable = [Some(true), None, Some(true), Some(false)]
            .into_iter()
            .collect::<BooleanArray<Nullable>>();
        assert_eq!(nullable.index(0), Some(Some(true)));
        assert_eq!(nullable.index(1), Some(None));
        assert_eq!(nullable.index(2), Some(Some(true)));
        assert_eq!(nullable.index(3), Some(Some(false)));
        assert_eq!(nullable.index(4), None);
    }

    #[test]
    fn buffer_ref_mut() {
        let input = [false, false, false, false];
        let mut array = input.iter().collect::<BooleanArray>();
        array.0.buffer_ref_mut()[0] = 0b0000_1111;
        assert_eq!(
            array.into_iter().collect::<Vec<_>>(),
            [true, true, true, true]
        );
    }

    #[test]
    fn convert_nullable() {
        let input = [true, false];
        let array = input.into_iter().collect::<BooleanArray>();
        let nullable: BooleanArray<Nullable> = array.into();
        assert!(nullable.all_valid());
        assert_eq!(
            nullable.into_iter().collect::<Vec<_>>(),
            [Some(true), Some(false)]
        );
    }

    #[test]
    fn size_of() {
        assert_eq!(mem::size_of::<BooleanArray>(), mem::size_of::<Bitmap>());
        assert_eq!(
            mem::size_of::<BooleanArray<Nullable>>(),
            mem::size_of::<BooleanArray>() + mem::size_of::<Bitmap>()
        );
    }
}
