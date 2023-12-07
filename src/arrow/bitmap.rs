//! Interop with [`arrow-rs`]'s bitmap.

use super::{buffer::ArrowBuffer, ArrowBufferBuilder};
use crate::{bitmap::Bitmap, buffer::BufferType};
use arrow_buffer::{BooleanBuffer, NullBuffer};

impl<Buffer: BufferType> From<Bitmap<Buffer>> for BooleanBuffer
where
    <Buffer as BufferType>::Buffer<u8>: Into<<ArrowBufferBuilder as BufferType>::Buffer<u8>>,
{
    fn from(value: Bitmap<Buffer>) -> Self {
        BooleanBuffer::new(value.buffer.into().finish(), value.offset, value.bits)
    }
}

impl<Buffer: BufferType> From<Bitmap<Buffer>> for NullBuffer
where
    <Buffer as BufferType>::Buffer<u8>: Into<<ArrowBufferBuilder as BufferType>::Buffer<u8>>,
{
    fn from(value: Bitmap<Buffer>) -> Self {
        Self::new(value.into())
    }
}

impl From<NullBuffer> for Bitmap<ArrowBuffer> {
    fn from(value: NullBuffer) -> Self {
        let bits = value.len();
        let offset = value.offset();
        Bitmap {
            buffer: value.into_inner().into_inner(),
            bits,
            offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Length;

    #[test]
    fn convert() {
        let input = vec![true, false, true];
        let bitmap_arrow = input.iter().collect::<Bitmap<ArrowBufferBuilder>>();
        assert_eq!(bitmap_arrow.len(), 3);
        let _: NullBuffer = bitmap_arrow.into();

        let bitmap = input.into_iter().collect::<Bitmap>();
        assert_eq!(bitmap.len(), 3);
        let _: BooleanBuffer = bitmap.into();
    }
}
