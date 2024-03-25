//! Interop with [`arrow-rs`] buffer types.

use std::{ptr::NonNull, sync::Arc};

use arrow_buffer::alloc::Allocation;

use crate::FixedSize;

mod boolean_buffer;
mod null_buffer;

mod buffer_builder;
pub use buffer_builder::BufferBuilder;
mod scalar_buffer;
pub use scalar_buffer::ScalarBuffer;

/// Arrow compatible buffer types.
pub trait Buffer<T: FixedSize>: crate::buffer::Buffer<T>
where
    Self: Sized + arrow_buffer::alloc::Allocation + 'static,
{
    /// Converts this buffer into an [`arrow_buffer::Buffer`].
    fn into_buffer(self) -> arrow_buffer::Buffer {
        let len = self.len();
        // Safety:
        // - Buffers are required to correctly implement length.
        unsafe {
            arrow_buffer::Buffer::from_custom_allocation(
                NonNull::new(self.as_bytes().as_ptr().cast_mut()).expect("buffer ptr is null"),
                len,
                Arc::new(self),
            )
        }
    }

    /// Converts this buffer into an [`arrow_buffer::ScalarBuffer<T>`].
    fn into_scalar_buffer(self) -> arrow_buffer::ScalarBuffer<T> {
        let len = self.len();
        arrow_buffer::ScalarBuffer::new(self.into_buffer(), 0, len)
    }
}

impl<T: FixedSize, U: Allocation + 'static> Buffer<T> for U where U: crate::buffer::Buffer<T> {}

/// Arrow compatible mutable buffer types.
pub trait BufferMut<T: FixedSize>: Buffer<T> + crate::buffer::BufferMut<T> {
    /// Copies this buffer into an [`arrow_buffer::MutableBuffer`].
    fn mutable_buffer(&self) -> arrow_buffer::MutableBuffer {
        let mut mutable_buffer = arrow_buffer::MutableBuffer::new(self.len());
        mutable_buffer.extend_from_slice(self.as_slice());
        mutable_buffer
    }

    /// Copies this buffer into an [`arrow_buffer::BufferBuilder<T>`].
    fn buffer_builder(&self) -> arrow_buffer::BufferBuilder<T> {
        arrow_buffer::BufferBuilder::new_from_buffer(self.mutable_buffer())
    }
}

impl<T: FixedSize, U: Allocation + 'static> BufferMut<T> for U where U: crate::buffer::BufferMut<T> {}
