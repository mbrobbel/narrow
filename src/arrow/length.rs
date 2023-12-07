//! Length implementations for [`arrow-rs`] items.

use crate::Length;
use arrow_buffer::{ArrowNativeType, Buffer, BufferBuilder, ScalarBuffer};

impl<T: ArrowNativeType> Length for BufferBuilder<T> {
    fn len(&self) -> usize {
        BufferBuilder::len(self)
    }
}

impl<T: ArrowNativeType> Length for ScalarBuffer<T> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

impl Length for Buffer {
    fn len(&self) -> usize {
        Buffer::len(self)
    }
}
