//! Interop with [`arrow-buffer`].

use crate::buffer::{Buffer, BufferMut, BufferType};
use crate::{FixedSize, Index};
use arrow_buffer::{BufferBuilder, ScalarBuffer};

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

/// A [`BufferType`] implementation for [`arrow_buffer::Buffer`].
#[derive(Clone, Copy)]
pub struct ArrowBuffer;

impl BufferType for ArrowBuffer {
    type Buffer<T: FixedSize> = arrow_buffer::Buffer;
}

impl<T: FixedSize> Buffer<T> for arrow_buffer::Buffer {
    fn as_slice(&self) -> &[T] {
        arrow_buffer::Buffer::typed_data(self)
    }
}

impl Index for arrow_buffer::Buffer {
    type Item<'a> = &'a u8
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.get_unchecked(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow() {
        let buffer = arrow_buffer::BufferBuilder::from_iter([1, 2, 3, 4]);
        assert_eq!(<_ as Buffer<u32>>::as_slice(&buffer), &[1, 2, 3, 4]);

        let mut buffer_builder = arrow_buffer::BufferBuilder::from_iter([1_u64, 2, 3, 4]);
        assert_eq!(
            <_ as BufferMut<u64>>::as_mut_slice(&mut buffer_builder),
            &[1, 2, 3, 4]
        );
        <_ as BufferMut<u64>>::as_mut_slice(&mut buffer_builder)[3] = 42;
        assert_eq!(
            <_ as Buffer<u64>>::as_slice(&buffer_builder),
            &[1, 2, 3, 42]
        );

        let scalar_buffer = arrow_buffer::ScalarBuffer::from_iter([1, 2, 3, 4]);
        assert_eq!(<_ as Buffer<u32>>::as_slice(&scalar_buffer), &[1, 2, 3, 4]);
    }
}
