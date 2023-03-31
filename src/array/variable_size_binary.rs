use crate::{
    bitmap::{Bitmap, ValidityBitmap},
    buffer::{Buffer, BufferRef},
    offset::{self, Offset},
    validity::Validity,
    Length,
};

pub struct VariableSizeBinaryArray<
    const NULLABLE: bool = false,
    DataBuffer = Vec<u8>,
    OffsetElement = i32,
    OffsetBuffer = Vec<OffsetElement>,
    BitmapBuffer = Vec<u8>,
>(Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>)
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>;

pub type BinaryArray<
    const NULLABLE: bool = false,
    DataBuffer = Vec<u8>,
    OffsetBuffer = Vec<i32>,
    BitmapBuffer = Vec<u8>,
> = VariableSizeBinaryArray<NULLABLE, DataBuffer, i32, OffsetBuffer, BitmapBuffer>;
pub type LargeBinaryArray<
    const NULLABLE: bool = false,
    DataBuffer = Vec<u8>,
    OffsetBuffer = Vec<i64>,
    BitmapBuffer = Vec<u8>,
> = VariableSizeBinaryArray<NULLABLE, DataBuffer, i64, OffsetBuffer, BitmapBuffer>;

impl<const NULLABLE: bool, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> Length
    for VariableSizeBinaryArray<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>: Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<const NULLABLE: bool, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> BufferRef
    for VariableSizeBinaryArray<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>: BufferRef,
{
    type Buffer = <Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> as BufferRef>::Buffer;
    type Element = <Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> as BufferRef>::Element;

    #[inline]
    fn buffer_ref(&self) -> &Self::Buffer {
        self.0.buffer_ref()
    }
}

impl<DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> ValidityBitmap
    for VariableSizeBinaryArray<true, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    BitmapBuffer: Buffer<u8>,
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement>,
{
    type Buffer = BitmapBuffer;

    #[inline]
    fn validity_bitmap(&self) -> &Bitmap<Self::Buffer> {
        self.0.validity_bitmap()
    }

    #[inline]
    fn validity_bitmap_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.validity_bitmap_mut()
    }
}

impl<T, const NULLABLE: bool, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> FromIterator<T>
    for VariableSizeBinaryArray<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>: FromIterator<T>,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<const NULLABLE: bool, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
    offset::buffer::OffsetBuffer<OffsetElement>
    for VariableSizeBinaryArray<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>:
        offset::buffer::OffsetBuffer<OffsetElement>,
{
    type Buffer = <Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> as
        offset::buffer::OffsetBuffer<OffsetElement>>::Buffer;

    #[inline]
    fn offset_buffer(&self) -> &Self::Buffer {
        self.0.offset_buffer()
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use offset::buffer::OffsetBuffer;

    use super::*;

    #[test]
    fn from_iter() {
        let input: [&[u8]; 4] = [&[1u8], &[2, 3], &[4, 5, 6], &[7, 8, 9, 0]];
        let array = input.into_iter().collect::<VariableSizeBinaryArray>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.buffer_ref(), &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);
        assert_eq!(array.offset_buffer(), &[0, 1, 3, 6, 10]);

        let input: [Option<&[u8]>; 4] = [Some(&[1u8]), None, Some(&[4, 5, 6]), Some(&[7, 8, 9, 0])];
        let array = input.into_iter().collect::<VariableSizeBinaryArray<true>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.buffer_ref(), &[1, 4, 5, 6, 7, 8, 9, 0]);
        assert_eq!(array.offset_buffer(), &[0, 1, 1, 4, 8]);
        assert_eq!(array.validity_bitmap().buffer_ref(), &[0b00001101]);
    }

    #[test]
    fn size_of() {
        assert_eq!(
            mem::size_of::<BinaryArray>(),
            mem::size_of::<Vec<u8>>() + mem::size_of::<Vec<i32>>()
        );
        assert_eq!(
            mem::size_of::<LargeBinaryArray>(),
            mem::size_of::<Vec<u8>>() + mem::size_of::<Vec<i64>>()
        );
        assert_eq!(
            mem::size_of::<BinaryArray<true>>(),
            mem::size_of::<Vec<u8>>() + mem::size_of::<Vec<i32>>() + mem::size_of::<Bitmap>()
        );
        assert_eq!(
            mem::size_of::<LargeBinaryArray<true>>(),
            mem::size_of::<Vec<u8>>() + mem::size_of::<Vec<i64>>() + mem::size_of::<Bitmap>()
        );
    }
}
