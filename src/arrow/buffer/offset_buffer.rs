//! Interop with [`arrow-rs`] offset buffer.

//! Interop with [`arrow-rs`] null buffer.

use arrow_buffer::{OffsetBuffer, ScalarBuffer};

use crate::{array::FixedSizePrimitiveArray, buffer::BufferType, offset::OffsetElement, Length};

impl<OffsetItem: OffsetElement> Length for OffsetBuffer<OffsetItem> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType>
    From<FixedSizePrimitiveArray<OffsetItem, false, Buffer>> for OffsetBuffer<OffsetItem>
where
    FixedSizePrimitiveArray<OffsetItem, false, Buffer>: Into<ScalarBuffer<OffsetItem>>,
{
    fn from(value: FixedSizePrimitiveArray<OffsetItem, false, Buffer>) -> Self {
        Self::new(value.into())
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> From<OffsetBuffer<OffsetItem>>
    for FixedSizePrimitiveArray<OffsetItem, false, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: From<ScalarBuffer<OffsetItem>>,
{
    fn from(value: OffsetBuffer<OffsetItem>) -> Self {
        Self(value.into_inner().into())
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        array::FixedSizePrimitiveArray, arrow::buffer::scalar_buffer::ArrowScalarBuffer,
        buffer::ArcBuffer,
    };

    use super::*;

    const INPUT: [usize; 4] = [1, 1, 2, 2];

    #[test]
    fn length() {
        let offset_buffer = OffsetBuffer::<i32>::from_lengths(INPUT);
        assert_eq!(Length::len(&offset_buffer), INPUT.len() + 1);
    }

    #[test]
    fn from() {
        let fixed_size_primitive_array = INPUT
            .into_iter()
            .map(|x| x.try_into().expect(""))
            .collect::<FixedSizePrimitiveArray<i32>>();
        assert_eq!(
            OffsetBuffer::<i32>::from(fixed_size_primitive_array).as_ref(),
            [1, 1, 2, 2]
        );

        let fixed_size_primitive_array_arc = INPUT
            .into_iter()
            .map(|x| x.try_into().expect(""))
            .collect::<FixedSizePrimitiveArray<i32, false, ArcBuffer>>(
        );
        assert_eq!(
            OffsetBuffer::<i32>::from(fixed_size_primitive_array_arc).as_ref(),
            [1, 1, 2, 2]
        );
    }

    #[test]
    fn into() {
        let offset_buffer = OffsetBuffer::<i64>::from_lengths(INPUT);
        assert_eq!(
            FixedSizePrimitiveArray::<i64, false, ArrowScalarBuffer>::from(offset_buffer.clone())
                .into_iter()
                .collect::<Vec<_>>(),
            offset_buffer.iter().collect::<Vec<_>>()
        );
    }
}
