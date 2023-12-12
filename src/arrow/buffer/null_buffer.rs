//! Interop with [`arrow-rs`] null buffer.

use arrow_buffer::{BooleanBuffer, NullBuffer};

use crate::{bitmap::Bitmap, buffer::BufferType, Length};

impl Length for NullBuffer {
    fn len(&self) -> usize {
        NullBuffer::len(self)
    }
}

impl<Buffer: BufferType> From<Bitmap<Buffer>> for NullBuffer
where
    Bitmap<Buffer>: Into<BooleanBuffer>,
{
    fn from(value: Bitmap<Buffer>) -> Self {
        Self::new(value.into())
    }
}

impl<Buffer: BufferType> From<NullBuffer> for Bitmap<Buffer>
where
    Bitmap<Buffer>: From<BooleanBuffer>,
{
    fn from(value: NullBuffer) -> Self {
        Bitmap::from(value.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use crate::{arrow::buffer::scalar_buffer::ArrowScalarBuffer, buffer::ArcBuffer};

    use super::*;

    const INPUT: [bool; 4] = [true, true, false, true];

    #[test]
    fn length() {
        let null_buffer = INPUT.into_iter().collect::<NullBuffer>();
        assert_eq!(Length::len(&null_buffer), INPUT.len());
    }

    #[test]
    fn from() {
        let bitmap = INPUT.into_iter().collect::<Bitmap>();
        assert_eq!(
            NullBuffer::from(bitmap).into_iter().collect::<Vec<_>>(),
            INPUT
        );

        let bitmap_arc = INPUT.into_iter().collect::<Bitmap<ArcBuffer>>();
        assert_eq!(
            NullBuffer::from(bitmap_arc).into_iter().collect::<Vec<_>>(),
            INPUT
        );
    }

    #[test]
    fn into() {
        let null_buffer = INPUT.into_iter().collect::<NullBuffer>();
        assert_eq!(
            Bitmap::<ArrowScalarBuffer>::from(null_buffer)
                .into_iter()
                .collect::<Vec<bool>>(),
            INPUT
        );
    }
}
