use crate::buffer::{Buffer, BufferMut, BufferType};
use crate::FixedSize;
use arrow_buffer::{ArrowNativeType, BufferBuilder, ScalarBuffer};

/// A [BufferType] implementation for [BufferBuilder].
pub struct ArrowBuffer;

impl BufferType for ArrowBuffer {
    type Buffer<T: FixedSize> = BufferBuilder<T>;
}

impl<T: FixedSize + ArrowNativeType> Buffer<T> for BufferBuilder<T> {
    fn as_slice(&self) -> &[T] {
        BufferBuilder::as_slice(self)
    }
}

impl<T: FixedSize + ArrowNativeType> BufferMut<T> for BufferBuilder<T> {
    fn as_mut_slice(&mut self) -> &mut [T] {
        BufferBuilder::as_slice_mut(self)
    }
}

/// A [BufferType] implementation for [ScalarBuffer].
pub struct ArrowScalarBuffer;

impl BufferType for ArrowScalarBuffer {
    type Buffer<T: FixedSize> = ScalarBuffer<T>;
}

impl<T: FixedSize + ArrowNativeType> Buffer<T> for ScalarBuffer<T> {
    fn as_slice(&self) -> &[T] {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow() {
        let buffer = arrow_buffer::BufferBuilder::from_iter([1, 2, 3, 4]);
        assert_eq!(<_ as Buffer<u32>>::as_slice(&buffer), &[1, 2, 3, 4]);

        let mut buffer = arrow_buffer::BufferBuilder::from_iter([1u64, 2, 3, 4]);
        assert_eq!(
            <_ as BufferMut<u64>>::as_mut_slice(&mut buffer),
            &[1, 2, 3, 4]
        );
        <_ as BufferMut<u64>>::as_mut_slice(&mut buffer)[3] = 42;
        assert_eq!(<_ as Buffer<u64>>::as_slice(&buffer), &[1, 2, 3, 42]);

        let buffer = arrow_buffer::ScalarBuffer::from_iter([1, 2, 3, 4]);
        assert_eq!(<_ as Buffer<u32>>::as_slice(&buffer), &[1, 2, 3, 4]);
    }
}
