//! Traits for memory buffers.

use crate::{
    bitmap::Bitmap,
    buffer::{Buffer, BufferMut},
    offset::OffsetValue,
    Primitive,
};

/// A validity bitmap.
pub trait ValidityBitmap<T>
where
    T: Buffer<u8>,
{
    fn validity_bitmap(&self) -> &Bitmap<T>;
}

/// A data buffer.
pub trait DataBuffer<T>
where
    T: Primitive,
{
    type Buffer: Buffer<T>;
    fn data_buffer(&self) -> &Self::Buffer;
}

/// A mutable data buffer.
pub trait DataBufferMut<T>
where
    T: Primitive,
{
    type Buffer: BufferMut<T>;
    fn data_buffer_mut(&mut self) -> &mut Self::Buffer;
}

/// An offset buffer.
pub trait OffsetBuffer<T>
where
    T: OffsetValue,
{
    type Buffer: Buffer<T>;
    fn offset_buffer(&self) -> &Self::Buffer;
}
