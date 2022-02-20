use crate::{Bitmap, Buffer, BufferMut, Primitive};

pub trait ValidityBitmap<T>
where
    T: Buffer<u8>,
{
    fn validity_bitmap(&self) -> &Bitmap<T>;
}

pub trait DataBuffer<T>
where
    T: Primitive,
{
    type Buffer: Buffer<T>;
    fn data_buffer(&self) -> &Self::Buffer;
}

pub trait DataBufferMut<T>
where
    T: Primitive,
{
    type Buffer: BufferMut<T>;
    fn data_buffer_mut(&mut self) -> &mut Self::Buffer;
}

pub trait OffsetBuffer<T>
where
    T: Primitive,
{
    type Buffer: Buffer<T>;
    fn offset_buffer(&self) -> &Self::Buffer;
}
