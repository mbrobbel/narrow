//! Interop with `arrow-rs` fixed-sized primitive array.

use crate::{
    array::FixedSizePrimitiveArray,
    arrow::{buffer::ArrowBufferBuilder, ArrowBuffer},
    bitmap::Bitmap,
    buffer::BufferType,
    nullable::Nullable,
    FixedSize, Length,
};
use arrow_array::{types::ArrowPrimitiveType, PrimitiveArray};
use arrow_buffer::{BooleanBuffer, NullBuffer, ScalarBuffer};

impl<T: FixedSize, U: ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<FixedSizePrimitiveArray<T, false, Buffer>> for PrimitiveArray<U>
where
    <Buffer as BufferType>::Buffer<T>: Length + Into<<ArrowBufferBuilder as BufferType>::Buffer<T>>,
{
    fn from(value: FixedSizePrimitiveArray<T, false, Buffer>) -> Self {
        let len = value.len();
        Self::new(ScalarBuffer::new(value.0.into().finish(), 0, len), None)
    }
}

impl<T: FixedSize, U: ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<FixedSizePrimitiveArray<T, true, Buffer>> for PrimitiveArray<U>
where
    <Buffer as BufferType>::Buffer<T>: Length + Into<<ArrowBufferBuilder as BufferType>::Buffer<T>>,
    Bitmap<Buffer>: Into<BooleanBuffer>,
{
    fn from(value: FixedSizePrimitiveArray<T, true, Buffer>) -> Self {
        let len = value.len();
        Self::new(
            ScalarBuffer::new(value.0.data.into().finish(), 0, len),
            Some(NullBuffer::new(value.0.validity.into())),
        )
    }
}

impl<T: ArrowPrimitiveType<Native = U>, U: FixedSize> From<PrimitiveArray<T>>
    for FixedSizePrimitiveArray<U, true, ArrowBuffer>
{
    fn from(value: PrimitiveArray<T>) -> Self {
        let (_, scala_buffer, opt_null_buffer) = value.into_parts();
        if let Some(null_buffer) = opt_null_buffer {
            Self(Nullable {
                data: scala_buffer.into_inner(),
                validity: null_buffer.into(),
            })
        } else {
            Self(Nullable::from(scala_buffer.into_inner()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::array::Int8Array;

    #[test]
    fn arrow_array() {
        use crate::{array::Int8Array, bitmap::ValidityBitmap};
        use arrow_array::{types::Int8Type, Array, PrimitiveArray};

        let input = [1, 2, 3, 4];
        let array_arrow_buffer = input
            .into_iter()
            .collect::<Int8Array<false, ArrowBufferBuilder>>();
        let array_arrow_from = PrimitiveArray::<Int8Type>::from(array_arrow_buffer);
        assert_eq!(array_arrow_from.len(), 4);

        let array =
            PrimitiveArray::<Int8Type>::from(input.into_iter().collect::<Int8Array<false>>());
        assert_eq!(array.len(), 4);

        let input_nullable = [Some(1), None, Some(3), Some(4)];
        let array_nullable = input_nullable
            .into_iter()
            .collect::<Int8Array<true, ArrowBufferBuilder>>();
        assert_eq!(array_nullable.null_count(), 1);
        let array_arrow = PrimitiveArray::<Int8Type>::from(array_nullable);
        assert_eq!(array_arrow.len(), 4);
        assert_eq!(array_arrow.null_count(), 1);
    }

    #[test]
    fn arrow_buffer() {
        let input = [1, 2, 3, 4];
        let mut array = input
            .into_iter()
            .collect::<Int8Array<false, ArrowBufferBuilder>>();
        assert_eq!(array.len(), 4);
        // Use arrow_buffer
        array.0.append_n(5, 5);
        assert_eq!(array.0.as_slice(), &[1, 2, 3, 4, 5, 5, 5, 5, 5]);

        let input_nullable = [Some(1), None, Some(3), Some(4)];
        let array_nullable = input_nullable
            .into_iter()
            .collect::<Int8Array<true, ArrowBufferBuilder>>();
        assert_eq!(array_nullable.len(), 4);
    }
}
