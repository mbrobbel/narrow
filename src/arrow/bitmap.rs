use super::buffer::ArrowBuffer;
use crate::{bitmap::Bitmap, buffer::BufferType};
use arrow_buffer::BooleanBuffer;

impl<Buffer: BufferType> From<Bitmap<Buffer>> for BooleanBuffer
where
    <Buffer as BufferType>::Buffer<u8>: Into<<ArrowBuffer as BufferType>::Buffer<u8>>,
{
    fn from(value: Bitmap<Buffer>) -> Self {
        BooleanBuffer::new(value.buffer.into().finish(), value.offset, value.bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Length;

    #[test]
    fn arrow_buffer() {
        let input = vec![true, false, true];
        let bitmap = input.into_iter().collect::<Bitmap<ArrowBuffer>>();
        assert_eq!(bitmap.len(), 3);

        let input = vec![true, false, true];
        let bitmap = input.into_iter().collect::<Bitmap<ArrowBuffer>>();
        assert_eq!(bitmap.len(), 3);
        assert_eq!(bitmap.into_iter().collect::<Vec<_>>(), [true, false, true]);
    }
}
