//! Interop with [`arrow-rs`] buffer builder.

use arrow_buffer::BufferBuilder;

use crate::{
    array::FixedSizePrimitiveArray,
    buffer::{Buffer, BufferMut, BufferType},
    FixedSize, Index, Length,
};

/// A [`BufferType`] implementation for [`BufferBuilder`].
#[derive(Clone, Copy)]
pub struct ArrowBufferBuilder;

impl BufferType for ArrowBufferBuilder {
    type Buffer<T: FixedSize> = BufferBuilder<T>;
}

impl<T: FixedSize> Buffer<T> for BufferBuilder<T> {
    fn as_slice(&self) -> &[T] {
        BufferBuilder::as_slice(self)
    }
}

impl<T: FixedSize> BufferMut<T> for BufferBuilder<T> {
    fn as_mut_slice(&mut self) -> &mut [T] {
        BufferBuilder::as_slice_mut(self)
    }
}

impl<T: FixedSize> Index for BufferBuilder<T> {
    type Item<'a> = &'a T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.as_slice().get_unchecked(index)
    }
}

impl<T: FixedSize> Length for BufferBuilder<T> {
    fn len(&self) -> usize {
        BufferBuilder::len(self)
    }
}

impl<T: FixedSize, Buffer: BufferType> From<FixedSizePrimitiveArray<T, false, Buffer>>
    for BufferBuilder<T>
{
    fn from(value: FixedSizePrimitiveArray<T, false, Buffer>) -> Self {
        // Note: this makes a copy
        let buffer = arrow_buffer::MutableBuffer::from(value.0.as_slice().to_vec());
        BufferBuilder::new_from_buffer(buffer)
    }
}

impl<T: FixedSize, Buffer: BufferType> From<BufferBuilder<T>>
    for FixedSizePrimitiveArray<T, false, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: From<arrow_buffer::Buffer>,
{
    fn from(mut value: BufferBuilder<T>) -> Self {
        FixedSizePrimitiveArray(value.finish().into())
    }
}

#[cfg(test)]
mod tests {
    use crate::{arrow::scalar_buffer::ArrowScalarBuffer, buffer::ArcBuffer};

    use super::*;

    const INPUT: [u32; 4] = [1, 2, 3, 4];

    #[test]
    fn length() {
        let buffer_builder = INPUT.into_iter().collect::<BufferBuilder<_>>();
        assert_eq!(Length::len(&buffer_builder), INPUT.len());
    }

    #[test]
    fn from() {
        let fixed_size_primitive_array = INPUT.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(
            BufferBuilder::from(fixed_size_primitive_array).as_slice(),
            INPUT
        );

        let fixed_size_primitive_array_arc =
            INPUT
                .into_iter()
                .collect::<FixedSizePrimitiveArray<_, false, ArcBuffer>>();
        assert_eq!(
            BufferBuilder::from(fixed_size_primitive_array_arc).as_slice(),
            INPUT
        );
    }

    #[test]
    fn into() {
        let buffer_builder = INPUT.into_iter().collect::<BufferBuilder<_>>();
        assert_eq!(
            FixedSizePrimitiveArray::<_, false, ArrowScalarBuffer>::from(buffer_builder)
                .into_iter()
                .copied()
                .collect::<Vec<_>>(),
            INPUT
        );
    }
}
