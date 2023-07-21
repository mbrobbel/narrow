use std::{
    iter::{self, FromIterator, Repeat, Take},
    marker::PhantomData,
};

use super::{Array, ArrayType};
use crate::{
    bitmap::{Bitmap, ValidityBitmap},
    buffer::Buffer,
    validity::Validity,
    Length,
};

/// A marker trait for unit types.
///
/// It is derived automatically for types without fields that have [NullArray]
/// as [ArrayType], and used as a trait bound on the methods that are used to
/// support deriving [Array] for these types.
///
/// # Safety
///
/// This trait is unsafe because the compiler can't verify that it only gets
/// implemented by unit types.
///
/// The [Default] implementation must return the only allowed value of this unit
/// type.
pub unsafe trait Unit
where
    Self: ArrayType + Copy + Default,
{
}

// # Safety:
// - std::mem::size_of::<()> == 0
unsafe impl Unit for () {}

/// A sequence of nulls.
///
/// This array type is also used as [ArrayType] when deriving [Array] for types
/// without fields ([Unit] types). The generic `T` is used to provide iterator
/// implementations for arrays of these unit types.
pub struct NullArray<T = (), const NULLABLE: bool = false, BitmapBuffer = Vec<u8>>(
    <Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>,
)
where
    T: Unit,
    Nulls<T>: Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>;

impl<T, const NULLABLE: bool, BitmapBuffer> Array for NullArray<T, NULLABLE, BitmapBuffer>
where
    T: Unit,
    Nulls<T>: Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
{
    type Item = T;
}

impl<T, const NULLABLE: bool, BitmapBuffer> Length for NullArray<T, NULLABLE, BitmapBuffer>
where
    T: Unit,
    Nulls<T>: Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    <Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>: Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T, BitmapBuffer> ValidityBitmap for NullArray<T, true, BitmapBuffer>
where
    T: Unit,
    BitmapBuffer: Buffer<u8>,
{
    type Buffer = BitmapBuffer;

    #[inline]
    fn validity_bitmap(&self) -> &Bitmap<Self::Buffer> {
        self.0.validity_bitmap()
    }
    #[inline]
    fn validity_bitmap_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.validity_bitmap_mut()
    }
}

// // TODO(mbrobbel): figure out why autotrait fails here
// unsafe impl<T, const NULLABLE: bool, BitmapBuffer> Send for NullArray<T,
// NULLABLE, BitmapBuffer> where
//     T: Unit,
//     Nulls<T>: Validity<NULLABLE>,
//     <Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>: Send,
// {
// }

// // TODO(mbrobbel): figure out why autotrait fails here
// unsafe impl<T, const NULLABLE: bool, BitmapBuffer> Sync for NullArray<T,
// NULLABLE, BitmapBuffer> where
//     T: Unit,
//     Nulls<T>: Validity<NULLABLE>,
//     <Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>: Sync,
// {
// }

impl<T, U, const NULLABLE: bool, BitmapBuffer> FromIterator<U>
    for NullArray<T, NULLABLE, BitmapBuffer>
where
    T: Unit,
    Nulls<T>: Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    <Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>: FromIterator<U>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = U>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<T, const NULLABLE: bool, BitmapBuffer> IntoIterator for NullArray<T, NULLABLE, BitmapBuffer>
where
    T: Unit,
    Nulls<T>: Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    <Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>: IntoIterator,
{
    type IntoIter =
        <<Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer> as IntoIterator>::IntoIter;
    type Item = <<Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// New type wrapper for null elements that implements Length.
#[derive(Debug, Copy, Clone, Default)]
pub struct Nulls<T> {
    /// The number of null elements
    len: usize,
    /// Covariant over `T`
    _ty: PhantomData<fn() -> T>,
}

impl<T> FromIterator<T> for Nulls<T>
where
    T: Unit,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            // TODO(mbrobbel): ExactSizeIterator
            len: iter.into_iter().count(),
            _ty: PhantomData,
        }
    }
}

impl<T> Extend<T> for Nulls<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.len += iter.into_iter().count();
    }
}

impl<T> IntoIterator for Nulls<T>
where
    T: Unit,
{
    type IntoIter = Take<Repeat<T>>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        iter::repeat(T::default()).take(self.len)
    }
}

impl<T> Length for Nulls<T> {
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;
    use crate::{
        bitmap::Bitmap,
        buffer::{Buffer, BufferRef},
        offset,
    };

    #[test]
    fn unit_types() {
        #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
        struct Foo;
        unsafe impl Unit for Foo {}
        impl ArrayType for Foo {
            type Array<
                DataBuffer: Buffer<Self::Primitive>,
                BitmapBuffer: Buffer<u8>,
                OffsetElement: offset::OffsetElement,
                OffsetBuffer: Buffer<OffsetElement>,
            > = NullArray<Foo, false, BitmapBuffer>;
            type Primitive = u8;
            type RefItem<'a> = &'a ();
        }
        let input = [Foo; 42];
        let array = input.into_iter().collect::<NullArray<Foo>>();
        assert_eq!(array.len(), 42);

        let input = [Some(Foo), None, Some(Foo), Some(Foo)];
        let array = input.into_iter().collect::<NullArray<Foo, true>>();
        assert_eq!(array.len(), 4);
        assert_eq!(input, array.into_iter().collect::<Vec<_>>().as_slice());
    }

    #[test]
    fn array_type() {
        let input = [(); 3];
        let array = input
            .iter()
            .copied()
            .collect::<<() as ArrayType>::Array<Vec<u8>, Vec<u8>, i32, Vec<i32>>>();
        assert_eq!(input, array.into_iter().collect::<Vec<_>>().as_slice());
        let input = [Some(()); 3];
        let array = input
            .iter()
            .copied()
            .collect::<<Option<()> as ArrayType>::Array<Vec<u8>, Vec<u8>, i32, Vec<i32>>>();
        assert_eq!(array.validity_bitmap().buffer_ref(), &[0b0000_0111]);
        assert_eq!(input, array.into_iter().collect::<Vec<_>>().as_slice());
    }

    #[test]
    fn into_iter() {
        let input = [(); 3];
        let array = input.iter().copied().collect::<NullArray>();
        assert_eq!(input, array.into_iter().collect::<Vec<_>>().as_slice());

        let input = [Some(()), None, Some(()), None];
        let array = input.iter().copied().collect::<NullArray<(), true>>();
        assert_eq!(input, array.into_iter().collect::<Vec<_>>().as_slice());
    }

    #[test]
    fn size_of() {
        assert_eq!(mem::size_of::<NullArray<()>>(), mem::size_of::<usize>());
        assert_eq!(
            mem::size_of::<NullArray<(), true>>(),
            mem::size_of::<NullArray<()>>() + mem::size_of::<Bitmap>()
        );
    }
}
