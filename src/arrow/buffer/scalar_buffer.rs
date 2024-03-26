//! Interop with [`arrow-rs`] scalar buffer.

use crate::{
    buffer::{Buffer, BufferType},
    FixedSize, Index, Length,
};

/// A [`BufferType`] implementation for [`ScalarBuffer`].
#[derive(Clone, Copy)]
pub struct ScalarBuffer;

impl BufferType for ScalarBuffer {
    type Buffer<T: FixedSize> = arrow_buffer::ScalarBuffer<T>;
}

impl<T: FixedSize> Buffer<T> for arrow_buffer::ScalarBuffer<T> {
    fn as_slice(&self) -> &[T] {
        self
    }
}

impl<T: FixedSize> Index for arrow_buffer::ScalarBuffer<T> {
    type Item<'a> = &'a T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.get_unchecked(index)
    }
}

impl<T: FixedSize> Length for arrow_buffer::ScalarBuffer<T> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }
}
