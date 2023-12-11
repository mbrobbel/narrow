//! Interop with [`arrow-rs`] scalar buffer.

use arrow_buffer::ScalarBuffer;

use crate::{
    array::FixedSizePrimitiveArray,
    buffer::{Buffer, BufferType},
    FixedSize, Index, Length,
};

/// A [`BufferType`] implementation for [`ScalarBuffer`].
#[derive(Clone, Copy)]
pub struct ArrowScalarBuffer;

impl BufferType for ArrowScalarBuffer {
    type Buffer<T: FixedSize> = ScalarBuffer<T>;
}

impl<T: FixedSize> Buffer<T> for ScalarBuffer<T> {
    fn as_slice(&self) -> &[T] {
        self
    }
}

impl<T: FixedSize> Index for ScalarBuffer<T> {
    type Item<'a> = &'a T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.get_unchecked(index)
    }
}

impl<T: FixedSize> Length for ScalarBuffer<T> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

impl<T: FixedSize, Buffer: BufferType> From<FixedSizePrimitiveArray<T, false, Buffer>>
    for ScalarBuffer<T>
where
    <Buffer as BufferType>::Buffer<T>: AsRef<[T]>,
{
    fn from(value: FixedSizePrimitiveArray<T, false, Buffer>) -> Self {
        let len = value.len();
        // Note: this makes a copy
        let buffer = arrow_buffer::Buffer::from_slice_ref(value.0.as_ref());
        ScalarBuffer::new(buffer, 0, len)
    }
}

impl<T: FixedSize, Buffer: BufferType> From<ScalarBuffer<T>>
    for FixedSizePrimitiveArray<T, false, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: From<arrow_buffer::Buffer>,
{
    fn from(value: ScalarBuffer<T>) -> Self {
        FixedSizePrimitiveArray(value.into_inner().into())
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::ArcBuffer;

    use super::*;

    const INPUT: [u32; 4] = [1, 2, 3, 4];

    #[test]
    fn length() {
        let scalar_buffer = INPUT.into_iter().collect::<ScalarBuffer<_>>();
        assert_eq!(Length::len(&scalar_buffer), INPUT.len());
    }

    #[test]
    fn from() {
        let fixed_size_primitive_array = INPUT.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(
            ScalarBuffer::from(fixed_size_primitive_array)
                .into_iter()
                .copied()
                .collect::<Vec<_>>(),
            INPUT
        );

        let fixed_size_primitive_array_arc =
            INPUT
                .into_iter()
                .collect::<FixedSizePrimitiveArray<_, false, ArcBuffer>>();
        assert_eq!(
            ScalarBuffer::from(fixed_size_primitive_array_arc)
                .into_iter()
                .copied()
                .collect::<Vec<_>>(),
            INPUT
        );
    }

    #[test]
    fn into() {
        let scalar_buffer = INPUT.into_iter().collect::<ScalarBuffer<_>>();
        assert_eq!(
            FixedSizePrimitiveArray::<_, false, ArrowScalarBuffer>::from(scalar_buffer)
                .into_iter()
                .copied()
                .collect::<Vec<_>>(),
            INPUT
        );
    }
}
