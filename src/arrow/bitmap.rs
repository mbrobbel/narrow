use super::buffer::ArrowBuffer;
use crate::{bitmap::Bitmap, buffer::BufferType};
use arrow_buffer::{BooleanBuffer, NullBuffer};

impl<Buffer: BufferType> From<Bitmap<Buffer>> for BooleanBuffer
where
    <Buffer as BufferType>::Buffer<u8>: Into<<ArrowBuffer as BufferType>::Buffer<u8>>,
{
    fn from(value: Bitmap<Buffer>) -> Self {
        BooleanBuffer::new(value.buffer.into().finish(), value.offset, value.bits)
    }
}

impl<Buffer: BufferType> From<Bitmap<Buffer>> for NullBuffer
where
    <Buffer as BufferType>::Buffer<u8>: Into<<ArrowBuffer as BufferType>::Buffer<u8>>,
{
    fn from(value: Bitmap<Buffer>) -> Self {
        Self::new(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Length;

    #[test]
    fn convert() {
        let input = vec![true, false, true];
        let bitmap = input.into_iter().collect::<Bitmap<ArrowBuffer>>();
        assert_eq!(bitmap.len(), 3);
        let _: NullBuffer = bitmap.into();

        let input = vec![true, false, true];
        let bitmap = input.into_iter().collect::<Bitmap>();
        assert_eq!(bitmap.len(), 3);
        let _: BooleanBuffer = bitmap.into();
    }
}
