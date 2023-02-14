use std::{marker::PhantomData, ops::Index};

use super::Array;
use crate::{
    bitmap::{Bitmap, ValidityBitmap},
    buffer::{Buffer, BufferRef, BufferRefMut},
    validity::Validity,
    Length, Primitive,
};

macro_rules! impl_primitive {
    ($ident:ident, $ty:ty) => {
        #[doc = "Array with ["]
        #[doc = stringify!($ty)]
        #[doc = "] values."]
        pub type $ident<
            const NULLABLE: bool = false,
            DataBuffer = Vec<$ty>,
            BitmapBuffer = Vec<u8>,
        > = FixedSizePrimitiveArray<$ty, NULLABLE, DataBuffer, BitmapBuffer>;
    };
}

impl_primitive!(Int8Array, i8);
impl_primitive!(Int16Array, i16);
impl_primitive!(Int32Array, i32);
impl_primitive!(Int64Array, i64);
impl_primitive!(Uint8Array, u8);
impl_primitive!(Uint16Array, u16);
impl_primitive!(Uint32Array, u32);
impl_primitive!(Uint64Array, u64);
impl_primitive!(Float32Array, f32);
impl_primitive!(Float64Array, f64);

/// Array with primitive values.
pub struct FixedSizePrimitiveArray<
    T,
    const NULLABLE: bool = false,
    DataBuffer = Vec<T>,
    BitmapBuffer = Vec<u8>,
>(
    <DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer>,
    PhantomData<fn() -> T>,
)
where
    T: Primitive,
    DataBuffer: Buffer<T> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>;

impl<T, const NULLABLE: bool, DataBuffer, BitmapBuffer> Array
    for FixedSizePrimitiveArray<T, NULLABLE, DataBuffer, BitmapBuffer>
where
    T: Primitive,
    DataBuffer: Buffer<T> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
{
    type Item = T;
}

impl<T, const NULLABLE: bool, DataBuffer, BitmapBuffer> BufferRef
    for FixedSizePrimitiveArray<T, NULLABLE, DataBuffer, BitmapBuffer>
where
    T: Primitive,
    DataBuffer: Buffer<T> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    <DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer>: BufferRef,
{
    type Buffer = <<DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer> as BufferRef>::Buffer;
    type Element =
        <<DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer> as BufferRef>::Element;

    fn buffer_ref(&self) -> &Self::Buffer {
        self.0.buffer_ref()
    }
}

impl<T, const NULLABLE: bool, DataBuffer, BitmapBuffer> Index<usize>
    for FixedSizePrimitiveArray<T, NULLABLE, DataBuffer, BitmapBuffer>
where
    T: Primitive,
    DataBuffer: Buffer<T> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    <DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer>: Index<usize>,
{
    type Output =
        <<DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer> as Index<usize>>::Output;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T, const NULLABLE: bool, DataBuffer, BitmapBuffer> BufferRefMut
    for FixedSizePrimitiveArray<T, NULLABLE, DataBuffer, BitmapBuffer>
where
    T: Primitive,
    DataBuffer: Buffer<T> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    <DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer>: BufferRefMut,
{
    type BufferMut =
        <<DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer> as BufferRefMut>::BufferMut;
    type Element =
        <<DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer> as BufferRefMut>::Element;

    fn buffer_ref_mut(&mut self) -> &mut Self::BufferMut {
        self.0.buffer_ref_mut()
    }
}

impl<T, const NULLABLE: bool, DataBuffer, BitmapBuffer> Length
    for FixedSizePrimitiveArray<T, NULLABLE, DataBuffer, BitmapBuffer>
where
    T: Primitive,
    DataBuffer: Buffer<T> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    <DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer>: Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T, DataBuffer, BitmapBuffer> ValidityBitmap
    for FixedSizePrimitiveArray<T, true, DataBuffer, BitmapBuffer>
where
    T: Primitive,
    DataBuffer: Buffer<T>,
    BitmapBuffer: Buffer<u8>,
{
    type Buffer = BitmapBuffer;

    fn validity_bitmap(&self) -> &Bitmap<Self::Buffer> {
        self.0.validity_bitmap()
    }
}

impl<T, const NULLABLE: bool, U, DataBuffer, BitmapBuffer> FromIterator<U>
    for FixedSizePrimitiveArray<T, NULLABLE, DataBuffer, BitmapBuffer>
where
    T: Primitive,
    DataBuffer: Buffer<T> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    <DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer>: FromIterator<U>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = U>,
    {
        Self(iter.into_iter().collect(), PhantomData)
    }
}

impl<'a, T, const NULLABLE: bool, DataBuffer, BitmapBuffer> IntoIterator
    for &'a FixedSizePrimitiveArray<T, NULLABLE, DataBuffer, BitmapBuffer>
where
    T: Primitive,
    DataBuffer: Buffer<T> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    &'a <DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer>: IntoIterator,
{
    type IntoIter =
        <&'a <DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer> as IntoIterator>::IntoIter;
    type Item =
        <&'a <DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, const NULLABLE: bool, DataBuffer, BitmapBuffer> IntoIterator
    for FixedSizePrimitiveArray<T, NULLABLE, DataBuffer, BitmapBuffer>
where
    T: Primitive,
    DataBuffer: Buffer<T> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    <DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer>: IntoIterator,
{
    type IntoIter =
        <<DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer> as IntoIterator>::IntoIter;
    type Item = <<DataBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::{mem, sync::Arc};

    use super::*;
    use crate::bitmap::Bitmap;

    #[test]
    fn from_iter() {
        let array = [1i8, 2, 3, 4].into_iter().collect::<Int8Array>();
        assert_eq!(array.len(), 4);

        let array = [1i8, 2, 3, 4]
            .iter()
            .copied()
            .collect::<Int8Array<false, Arc<[i8]>>>();
        assert_eq!(array.len(), 4);

        let array = [Some(1u8), None, Some(3), Some(4)]
            .iter()
            .map(|opt| opt.as_ref().copied())
            .collect::<Uint8Array<true>>();
        assert_eq!(array.len(), 4);
    }

    #[test]
    fn into_iter() {
        let input = [1u64, 2, 3, 4, 5, 6, 7];
        let array = input.iter().copied().collect::<Uint64Array>();
        let output = (&array).into_iter().copied().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let input = [3f32, 1., 4.];
        let array = input.iter().copied().collect::<Float32Array>();
        let output = (&array).into_iter().copied().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let input = [Some(1u8), None, Some(3), Some(4)];
        let array = input.iter().copied().collect::<Uint8Array<true>>();
        let output = (&array)
            .into_iter()
            .map(|opt| opt.copied())
            .collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());
    }

    #[test]
    fn buffers() {
        let input = [1u64, 2, 3, 4, 5, 6, 7];
        let array = input.iter().copied().collect::<Uint64Array>();
        assert_eq!(array.buffer_ref().as_slice(), input);

        let input = [Some(1u8), None, Some(3), Some(4)];
        let array = input.iter().copied().collect::<Uint8Array<true>>();
        assert_eq!(array.buffer_ref().as_slice(), [1u8, u8::default(), 3, 4]);
        assert_eq!(
            array.validity_bitmap(),
            [true, false, true, true].as_slice()
        );
    }

    #[test]
    fn size_of() {
        assert_eq!(mem::size_of::<Int8Array>(), mem::size_of::<Vec<i8>>());
        assert_eq!(
            std::mem::size_of::<Int8Array<true>>(),
            mem::size_of::<Int8Array>() + mem::size_of::<Bitmap>()
        );
    }
}
