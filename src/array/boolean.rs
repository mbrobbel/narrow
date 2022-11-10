use super::Array;
use crate::{
    bitmap::{Bitmap, ValidityBitmap},
    buffer::{Buffer, BufferRef, BufferRefMut},
    validity::Validity,
    Length,
};

/// Array with boolean values.
///
/// Values are stored using single bits in a [Bitmap].
pub struct BooleanArray<const NULLABLE: bool = false, DataBuffer = Vec<u8>, BitmapBuffer = Vec<u8>>(
    <Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer>,
)
where
    DataBuffer: Buffer<u8>,
    BitmapBuffer: Buffer<u8>,
    Bitmap<DataBuffer>: Validity<NULLABLE>;

impl<const NULLABLE: bool, DataBuffer, BitmapBuffer> Array
    for BooleanArray<NULLABLE, DataBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    BitmapBuffer: Buffer<u8>,
    Bitmap<DataBuffer>: Validity<NULLABLE>,
{
    type Item = bool;
}

impl<const NULLABLE: bool, DataBuffer, BitmapBuffer> BufferRef
    for BooleanArray<NULLABLE, DataBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    BitmapBuffer: Buffer<u8>,
    Bitmap<DataBuffer>: Validity<NULLABLE>,
    <Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer>: BufferRef,
{
    type Buffer =
        <<Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer> as BufferRef>::Buffer;
    type Element =
        <<Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer> as BufferRef>::Element;

    fn buffer_ref(&self) -> &Self::Buffer {
        self.0.buffer_ref()
    }
}

impl<const NULLABLE: bool, DataBuffer, BitmapBuffer> BufferRefMut
    for BooleanArray<NULLABLE, DataBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    BitmapBuffer: Buffer<u8>,
    Bitmap<DataBuffer>: Validity<NULLABLE>,
    <Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer>: BufferRefMut,
{
    type BufferMut =
        <<Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer> as BufferRefMut>::BufferMut;
    type Element =
        <<Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer> as BufferRefMut>::Element;

    fn buffer_ref_mut(&mut self) -> &mut Self::BufferMut {
        self.0.buffer_ref_mut()
    }
}

impl<const NULLABLE: bool, DataBuffer, BitmapBuffer> Length
    for BooleanArray<NULLABLE, DataBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    BitmapBuffer: Buffer<u8>,
    Bitmap<DataBuffer>: Validity<NULLABLE>,
    <Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer>: Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<DataBuffer, BitmapBuffer> ValidityBitmap for BooleanArray<true, DataBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    BitmapBuffer: Buffer<u8>,
{
    type Buffer = BitmapBuffer;

    #[inline]
    fn validity_bitmap(&self) -> &Bitmap<BitmapBuffer> {
        self.0.validity_bitmap()
    }
}

impl<T, const NULLABLE: bool, DataBuffer, BitmapBuffer> FromIterator<T>
    for BooleanArray<NULLABLE, DataBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    BitmapBuffer: Buffer<u8>,
    Bitmap<DataBuffer>: Validity<NULLABLE>,
    <Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer>: FromIterator<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a, const NULLABLE: bool, DataBuffer, BitmapBuffer> IntoIterator
    for &'a BooleanArray<NULLABLE, DataBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    BitmapBuffer: Buffer<u8>,
    Bitmap<DataBuffer>: Validity<NULLABLE>,
    &'a <Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer>: IntoIterator,
{
    type IntoIter =
        <&'a <Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer> as
IntoIterator>::IntoIter;
    type Item =
        <&'a <Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const NULLABLE: bool, DataBuffer, BitmapBuffer> IntoIterator
    for BooleanArray<NULLABLE, DataBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    BitmapBuffer: Buffer<u8>,
    Bitmap<DataBuffer>: Validity<NULLABLE>,
    <Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer>: IntoIterator,
{
    type IntoIter =
        <<Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer> as
IntoIterator>::IntoIter;
    type Item =
        <<Bitmap<DataBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn from_iter() {
        let array = [true, false, true, true]
            .into_iter()
            .collect::<BooleanArray<false, Box<[u8]>, Box<[u8]>>>();
        assert_eq!(array.len(), 4);

        let array = [Some(true), None, Some(true), Some(false)]
            .into_iter()
            .collect::<BooleanArray<true>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.is_valid(0), Some(true));
        assert_eq!(array.is_null(1), Some(true));
        assert_eq!(array.is_valid(2), Some(true));
        assert_eq!(array.is_valid(3), Some(true));
        assert!(array.is_valid(4).is_none());
        assert_eq!(array.validity_bitmap().len(), array.len());
    }

    #[test]
    fn into_iter() {
        let input = [true, false, true, true];
        let array = input.iter().collect::<BooleanArray>();
        let output = (&array).into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let input = [Some(true), None, Some(true), Some(false)];
        let array = input.into_iter().collect::<BooleanArray<true>>();
        let output = (&array).into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());
    }

    #[test]
    fn buffer_mut() {
        let input = [false, false, false, false];
        let mut array = input.iter().collect::<BooleanArray>();
        array.buffer_ref_mut()[0] = 0b0000_1111;
        assert_eq!(
            array.into_iter().collect::<Vec<_>>(),
            [true, true, true, true]
        );
    }

    #[test]
    fn size_of() {
        assert_eq!(mem::size_of::<BooleanArray>(), mem::size_of::<Bitmap>());
        assert_eq!(
            mem::size_of::<BooleanArray<true>>(),
            mem::size_of::<BooleanArray>() + mem::size_of::<Bitmap>()
        );
    }
}
