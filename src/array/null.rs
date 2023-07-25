//! A sequence of nulls.

use super::{Array, ArrayType};
use crate::{
    buffer::{BufferType, VecBuffer},
    validity::Validity,
    Length,
};
use std::{
    iter::{self, Repeat, Take},
    marker::PhantomData,
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

pub struct NullArray<
    T: Unit = (),
    const NULLABLE: bool = false,
    BitmapBuffer: BufferType = VecBuffer,
>(<Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>)
where
    Nulls<T>: Validity<NULLABLE>;

impl<T: Unit, const NULLABLE: bool, BitmapBuffer: BufferType> Array
    for NullArray<T, NULLABLE, BitmapBuffer>
where
    Nulls<T>: Validity<NULLABLE>,
{
}

impl<T: Unit, const NULLABLE: bool, BitmapBuffer: BufferType> Default
    for NullArray<T, NULLABLE, BitmapBuffer>
where
    Nulls<T>: Validity<NULLABLE>,
    <Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Unit, U, const NULLABLE: bool, BitmapBuffer: BufferType> Extend<U>
    for NullArray<T, NULLABLE, BitmapBuffer>
where
    Nulls<T>: Validity<NULLABLE>,
    <Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<T: Unit, U, const NULLABLE: bool, BitmapBuffer: BufferType> FromIterator<U>
    for NullArray<T, NULLABLE, BitmapBuffer>
where
    Nulls<T>: Validity<NULLABLE>,
    <Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>: FromIterator<U>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = U>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<T: Unit, const NULLABLE: bool, BitmapBuffer: BufferType> Length
    for NullArray<T, NULLABLE, BitmapBuffer>
where
    Nulls<T>: Validity<NULLABLE>,
    <Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Unit, const NULLABLE: bool, BitmapBuffer: BufferType> IntoIterator
    for NullArray<T, NULLABLE, BitmapBuffer>
where
    Nulls<T>: Validity<NULLABLE>,
    <Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>: IntoIterator,
{
    type Item = <<Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer> as IntoIterator>::Item;
    type IntoIter =
        <<Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// TODO(mbrobbel): figure out why autotrait fails here
unsafe impl<T: Unit, const NULLABLE: bool, BitmapBuffer: BufferType> Send
    for NullArray<T, NULLABLE, BitmapBuffer>
where
    Nulls<T>: Validity<NULLABLE>,
    <Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>: Send,
{
}

// TODO(mbrobbel): figure out why autotrait fails here
unsafe impl<T: Unit, const NULLABLE: bool, BitmapBuffer: BufferType> Sync
    for NullArray<T, NULLABLE, BitmapBuffer>
where
    Nulls<T>: Validity<NULLABLE>,
    <Nulls<T> as Validity<NULLABLE>>::Storage<BitmapBuffer>: Sync,
{
}

/// New type wrapper for null elements that implements Length.
#[derive(Debug, Copy, Clone, Default)]
pub struct Nulls<T: Unit> {
    /// The number of null elements
    len: usize,

    /// Covariant over `T`
    _ty: PhantomData<fn() -> T>,
}

impl<T: Unit> FromIterator<T> for Nulls<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            // TODO(mbrobbel): ExactSizeIterator
            len: iter.into_iter().count(),
            _ty: PhantomData,
        }
    }
}

impl<T: Unit> Extend<T> for Nulls<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.len += iter.into_iter().count();
    }
}

impl<T: Unit> IntoIterator for Nulls<T> {
    type IntoIter = Take<Repeat<T>>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        iter::repeat(T::default()).take(self.len)
    }
}

impl<T: Unit> Length for Nulls<T> {
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitmap::Bitmap;
    use std::mem;

    #[test]
    fn unit_types() {
        #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
        struct Foo;
        unsafe impl Unit for Foo {}
        impl ArrayType for Foo {
            type Array<Buffer: BufferType> = NullArray<Foo, false, Buffer>;
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
    fn into_iter() {
        let input = [(); 3];
        let array = input.iter().copied().collect::<NullArray>();
        assert_eq!(input, array.into_iter().collect::<Vec<_>>().as_slice());

        let input = [Some(()), None, Some(()), None];
        let array = input.iter().copied().collect::<NullArray<_, true>>();
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
