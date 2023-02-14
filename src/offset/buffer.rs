use super::OffsetElement;
use crate::buffer::Buffer;

/// An offset buffer.
pub trait OffsetBuffer<T>
where
    T: OffsetElement,
{
    type Buffer: ?Sized + Buffer<T>;

    fn offset_buffer(&self) -> &Self::Buffer;
}
