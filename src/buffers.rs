use crate::{Bitmap, Buffer};

pub trait ValidityBitmap<const A: usize> {
    fn validity_bitmap(&self) -> &Bitmap<A>;
}

pub trait DataBuffer<T, const A: usize> {
    fn data_buffer(&self) -> &Buffer<T, A>;
}

pub trait OffsetBuffer<T, const A: usize> {
    fn offset_buffer(&self) -> &Buffer<T, A>;
}
