//! Interop with [`arrow-rs`] boolean buffer.

use arrow_buffer::BooleanBuffer;

use crate::{bitmap::Bitmap, buffer::BufferType, Length};

impl Length for BooleanBuffer {
    fn len(&self) -> usize {
        BooleanBuffer::len(self)
    }
}

impl<Buffer: BufferType> From<Bitmap<Buffer>> for BooleanBuffer
where
    <Buffer as BufferType>::Buffer<u8>: Into<arrow_buffer::Buffer>,
{
    fn from(value: Bitmap<Buffer>) -> Self {
        Self::new(value.buffer.into(), value.offset, value.bits)
    }
}

impl<Buffer: BufferType> From<BooleanBuffer> for Bitmap<Buffer>
where
    <Buffer as BufferType>::Buffer<u8>: From<arrow_buffer::Buffer>,
{
    fn from(value: BooleanBuffer) -> Self {
        let bits = value.len();
        let offset = value.offset();
        Bitmap {
            buffer: value.into_inner().into(),
            bits,
            offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{arrow::buffer::scalar_buffer::ArrowScalarBuffer, buffer::ArcBuffer};

    use super::*;

    const INPUT: [bool; 4] = [true, true, false, true];

    #[test]
    fn length() {
        let boolean_buffer = INPUT.into_iter().collect::<BooleanBuffer>();
        assert_eq!(Length::len(&boolean_buffer), INPUT.len());
    }

    #[test]
    fn from() {
        let bitmap = INPUT.into_iter().collect::<Bitmap>();
        assert_eq!(
            BooleanBuffer::from(bitmap).into_iter().collect::<Vec<_>>(),
            INPUT
        );

        let bitmap_arc = INPUT.into_iter().collect::<Bitmap<ArcBuffer>>();
        assert_eq!(
            BooleanBuffer::from(bitmap_arc)
                .into_iter()
                .collect::<Vec<_>>(),
            INPUT
        );
    }

    #[test]
    fn into() {
        let boolean_buffer = INPUT.into_iter().collect::<BooleanBuffer>();
        assert_eq!(
            Bitmap::<ArrowScalarBuffer>::from(boolean_buffer)
                .into_iter()
                .collect::<Vec<bool>>(),
            INPUT
        );
    }
}
