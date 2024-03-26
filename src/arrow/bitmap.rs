//! Interop with [`arrow-rs`] null buffer for bitmaps.

use crate::{bitmap::Bitmap, buffer::BufferType, Length};

impl<Buffer: BufferType> From<Bitmap<Buffer>> for arrow_buffer::BooleanBuffer
where
    <Buffer as BufferType>::Buffer<u8>: Into<arrow_buffer::Buffer>,
{
    fn from(value: Bitmap<Buffer>) -> Self {
        Self::new(value.buffer.into(), value.offset, value.bits)
    }
}

impl<Buffer: BufferType> From<Bitmap<Buffer>> for arrow_buffer::NullBuffer
where
    arrow_buffer::BooleanBuffer: From<Bitmap<Buffer>>,
{
    fn from(value: Bitmap<Buffer>) -> Self {
        Self::new(value.into())
    }
}

impl<Buffer: BufferType> From<arrow_buffer::BooleanBuffer> for Bitmap<Buffer>
where
    <Buffer as BufferType>::Buffer<u8>: From<arrow_buffer::ScalarBuffer<u8>>,
{
    fn from(value: arrow_buffer::BooleanBuffer) -> Self {
        let bits = value.len();
        let offset = value.offset();
        let buffer = value.into_inner();
        let len = buffer.len();
        Bitmap {
            buffer: arrow_buffer::ScalarBuffer::new(buffer, 0, len).into(),
            bits,
            offset,
        }
    }
}

impl<Buffer: BufferType> From<arrow_buffer::NullBuffer> for Bitmap<Buffer>
where
    Bitmap<Buffer>: From<arrow_buffer::BooleanBuffer>,
{
    fn from(value: arrow_buffer::NullBuffer) -> Self {
        Bitmap::from(value.into_inner())
    }
}

impl<Buffer: BufferType> PartialEq<Bitmap<Buffer>> for arrow_buffer::BooleanBuffer {
    fn eq(&self, other: &Bitmap<Buffer>) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(a, b)| a == b)
    }
}

impl<Buffer: BufferType> PartialEq<Bitmap<Buffer>> for arrow_buffer::NullBuffer {
    fn eq(&self, other: &Bitmap<Buffer>) -> bool {
        self.inner().eq(other)
    }
}

impl<Buffer: BufferType> PartialEq<arrow_buffer::BooleanBuffer> for Bitmap<Buffer> {
    fn eq(&self, other: &arrow_buffer::BooleanBuffer) -> bool {
        other.eq(self)
    }
}

impl<Buffer: BufferType> PartialEq<arrow_buffer::NullBuffer> for Bitmap<Buffer> {
    fn eq(&self, other: &arrow_buffer::NullBuffer) -> bool {
        other.eq(self)
    }
}

#[cfg(test)]
mod test {
    use crate::buffer::{ArcBuffer, BoxBuffer, VecBuffer};

    use super::*;

    const INPUT: [bool; 5] = [true, false, true, false, true];

    #[test]
    fn convert() {
        fn from<Buffer: BufferType>()
        where
            Bitmap<Buffer>: FromIterator<bool>
                + Into<arrow_buffer::BooleanBuffer>
                + Into<arrow_buffer::NullBuffer>,
        {
            let boolean_buffer: arrow_buffer::BooleanBuffer =
                INPUT.into_iter().collect::<Bitmap<Buffer>>().into();
            let null_buffer: arrow_buffer::NullBuffer =
                INPUT.into_iter().collect::<Bitmap<Buffer>>().into();
            let bitmap = INPUT.into_iter().collect::<Bitmap<Buffer>>();
            assert_eq!(bitmap, boolean_buffer);
            assert_eq!(bitmap, null_buffer);
        }
        fn into<Buffer: BufferType>()
        where
            Bitmap<Buffer>: From<arrow_buffer::BooleanBuffer> + From<arrow_buffer::NullBuffer>,
        {
            let boolean_buffer = INPUT.into_iter().collect::<arrow_buffer::BooleanBuffer>();
            let null_buffer = INPUT.into_iter().collect::<arrow_buffer::NullBuffer>();
            assert_eq!(
                Bitmap::<Buffer>::from(boolean_buffer.clone()),
                boolean_buffer
            );
            assert_eq!(Bitmap::<Buffer>::from(null_buffer.clone()), null_buffer);
        }
        from::<VecBuffer>();
        from::<ArcBuffer>();
        from::<BoxBuffer>();
        from::<crate::arrow::buffer::ScalarBuffer>();
        from::<crate::arrow::buffer::BufferBuilder>();

        into::<VecBuffer>();
        // into::<ArcBuffer>(); missing ScalarBuffer<u8> from Arc<[u8]>
        // into::<BoxBuffer>(); missing ScalarBuffer<u8> from Box<[u8]>
        into::<crate::arrow::buffer::ScalarBuffer>();
        // into::<crate::arrow::buffer::BufferBuilder>(); missing BufferBuilder<u8> from ScalarBuffer<u8>
    }
}
