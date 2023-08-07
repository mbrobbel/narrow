use crate::{
    array::FixedSizePrimitiveArray, arrow::buffer::ArrowBuffer, bitmap::Bitmap, buffer::BufferType,
    FixedSize, Length,
};
use arrow_array::{types::ArrowPrimitiveType, PrimitiveArray};
use arrow_buffer::{BooleanBuffer, NullBuffer, ScalarBuffer};

impl<T: FixedSize, U: ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<FixedSizePrimitiveArray<T, false, Buffer>> for PrimitiveArray<U>
where
    <Buffer as BufferType>::Buffer<T>: Length + Into<<ArrowBuffer as BufferType>::Buffer<T>>,
{
    fn from(value: FixedSizePrimitiveArray<T, false, Buffer>) -> Self {
        let len = value.len();
        Self::new(ScalarBuffer::new(value.0.into().finish(), 0, len), None)
    }
}

impl<T: FixedSize, U: ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<FixedSizePrimitiveArray<T, true, Buffer>> for PrimitiveArray<U>
where
    <Buffer as BufferType>::Buffer<T>: Length + Into<<ArrowBuffer as BufferType>::Buffer<T>>,
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::array::Int8Array;

    #[test]
    #[cfg(feature = "arrow-array")]
    fn arrow_array() {
        use crate::{array::Int8Array, bitmap::ValidityBitmap};
        use arrow_array::{types::Int8Type, Array, PrimitiveArray};

        let input = [1, 2, 3, 4];
        let array = input.into_iter().collect::<Int8Array<false, ArrowBuffer>>();
        let array = PrimitiveArray::<Int8Type>::from(array);
        assert_eq!(array.len(), 4);

        let input = [1, 2, 3, 4];
        let array = input.into_iter().collect::<Int8Array<false>>();
        let array = PrimitiveArray::<Int8Type>::from(array);
        assert_eq!(array.len(), 4);

        let input = [Some(1), None, Some(3), Some(4)];
        let array = input.into_iter().collect::<Int8Array<true, ArrowBuffer>>();
        assert_eq!(array.null_count(), 1);
        let array = PrimitiveArray::<Int8Type>::from(array);
        assert_eq!(array.len(), 4);
        assert_eq!(array.null_count(), 1);
    }

    #[test]
    fn arrow_buffer() {
        let input = [1, 2, 3, 4];
        let mut array = input.into_iter().collect::<Int8Array<false, ArrowBuffer>>();
        assert_eq!(array.len(), 4);
        // Use arrow_buffer
        array.0.append_n(5, 5);
        assert_eq!(array.0.as_slice(), &[1, 2, 3, 4, 5, 5, 5, 5, 5]);

        let input = [Some(1), None, Some(3), Some(4)];
        let array = input.into_iter().collect::<Int8Array<true, ArrowBuffer>>();
        assert_eq!(array.len(), 4);
    }
}
