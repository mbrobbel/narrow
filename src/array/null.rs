//! A sequence of nulls.

use super::{Array, ArrayType};
use crate::{
    Index, Length,
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{BufferType, VecBuffer},
    nullability::{NonNullable, Nullability, Nullable},
    validity::Validity,
};
use std::{
    iter::{self, Repeat, Take},
    marker::PhantomData,
};

/// A marker trait for unit types.
///
/// It is derived automatically for types without fields that have [`NullArray`]
/// as [`ArrayType`], and used as a trait bound on the methods that are used to
/// support deriving [Array] for these types.
///
/// # Safety
///
/// This trait is unsafe because the compiler can't verify that it only gets
/// implemented by unit types.
pub unsafe trait Unit
where
    Self: Default + Sized,
{
    /// This is the item that is returned
    type Item: ArrayType<Self> + Copy + From<Self> + Send + Sync + 'static;
}

// # Safety:
// - std::mem::size_of::<()> == 0
unsafe impl Unit for () {
    type Item = Self;
}

/// A sequence of nulls.
pub struct NullArray<
    T: Unit = (),
    Nullable: Nullability = NonNullable,
    Buffer: BufferType = VecBuffer,
>(pub(crate) Nullable::Collection<Nulls<T>, Buffer>);

impl<T: Unit, Nullable: Nullability, Buffer: BufferType> Array for NullArray<T, Nullable, Buffer> {
    type Item = Nullable::Item<T>;
}

impl<T: Unit, Nullable: Nullability, Buffer: BufferType> Clone for NullArray<T, Nullable, Buffer>
where
    Nullable::Collection<Nulls<T>, Buffer>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Unit, Nullable: Nullability, Buffer: BufferType> Default for NullArray<T, Nullable, Buffer>
where
    Nullable::Collection<Nulls<T>, Buffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Unit, U, Nullable: Nullability, Buffer: BufferType> Extend<U>
    for NullArray<T, Nullable, Buffer>
where
    Nullable::Collection<Nulls<T>, Buffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: Unit, Buffer: BufferType> From<NullArray<T, NonNullable, Buffer>>
    for NullArray<T, Nullable, Buffer>
where
    Bitmap<Buffer>: FromIterator<bool>,
{
    fn from(value: NullArray<T, NonNullable, Buffer>) -> Self {
        Self(Validity::from(value.0))
    }
}

impl<T: Unit, U, Nullable: Nullability, Buffer: BufferType> FromIterator<U>
    for NullArray<T, Nullable, Buffer>
where
    Nullable::Collection<Nulls<T>, Buffer>: FromIterator<U>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = U>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<T: Unit, Nullable: Nullability, Buffer: BufferType> Index for NullArray<T, Nullable, Buffer>
where
    Nullable::Collection<Nulls<T>, Buffer>: Index,
{
    type Item<'a>
        = <Nullable::Collection<Nulls<T>, Buffer> as Index>::Item<'a>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.0.index_unchecked(index)
    }
}

impl<T: Unit, Nullable: Nullability, Buffer: BufferType> Length for NullArray<T, Nullable, Buffer>
where
    Nullable::Collection<Nulls<T>, Buffer>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Unit, Nullable: Nullability, Buffer: BufferType> IntoIterator
    for NullArray<T, Nullable, Buffer>
where
    Nullable::Collection<Nulls<T>, Buffer>: IntoIterator,
{
    type Item = <Nullable::Collection<Nulls<T>, Buffer> as IntoIterator>::Item;
    type IntoIter = <Nullable::Collection<Nulls<T>, Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Unit, Buffer: BufferType> BitmapRef for NullArray<T, Nullable, Buffer> {
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<T: Unit, Buffer: BufferType> BitmapRefMut for NullArray<T, Nullable, Buffer> {
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<T: Unit, Buffer: BufferType> ValidityBitmap for NullArray<T, Nullable, Buffer> {}

/// New type wrapper for null elements that implements Length.
#[derive(Debug, Copy, Clone, Default)]
pub struct Nulls<T: Unit> {
    /// The number of null elements
    len: usize,

    /// Covariant over `T`
    _ty: PhantomData<fn() -> T>,
}

impl<T: Unit> Nulls<T> {
    #[cfg(feature = "arrow-rs")]
    /// Constructs a Nulls from a given length.
    pub(crate) fn new(len: usize) -> Self {
        Self {
            len,
            _ty: PhantomData,
        }
    }
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
        self.len = self
            .len
            .checked_add(iter.into_iter().count())
            .expect("len overflow");
    }
}

impl<T: Unit> Index for Nulls<T> {
    type Item<'a>
        = T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, _index: usize) -> Self::Item<'_> {
        T::default()
    }
}

impl<T: Unit> IntoIterator for Nulls<T> {
    type IntoIter = Take<Repeat<T::Item>>;
    type Item = T::Item;

    fn into_iter(self) -> Self::IntoIter {
        iter::repeat(T::default().into()).take(self.len)
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
    use crate::{array::UnionType, offset::Offset};
    use std::mem;

    #[test]
    fn unit_types() {
        #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
        struct Foo;
        /// Safety:
        /// - Foo is a unit struct.
        unsafe impl Unit for Foo {
            type Item = Self;
        }
        impl ArrayType<Self> for Foo {
            type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
                NullArray<Foo, NonNullable, Buffer>;
        }
        let input = [Foo; 42];
        let array = input.into_iter().collect::<NullArray<Foo>>();
        assert_eq!(array.len(), 42);

        let input_nullable = [Some(Foo), None, Some(Foo), Some(Foo)];
        let array_nullable = input_nullable
            .into_iter()
            .collect::<NullArray<Foo, Nullable>>();
        assert_eq!(array_nullable.len(), 4);
        assert_eq!(array_nullable.index(0), Some(Some(Foo)));
        assert_eq!(array_nullable.index(1), Some(None));
        assert_eq!(array_nullable.index(2), Some(Some(Foo)));
        assert_eq!(array_nullable.index(4), None);
        assert_eq!(
            input_nullable,
            array_nullable.into_iter().collect::<Vec<_>>().as_slice()
        );
    }

    #[test]
    fn into_iter() {
        let input = [(); 3];
        let array = input.iter().copied().collect::<NullArray>();
        assert_eq!(input, array.into_iter().collect::<Vec<_>>().as_slice());
    }

    #[test]
    fn index() {
        let array = [(); 1].iter().copied().collect::<NullArray>();
        assert_eq!(array.index(0), Some(()));
        assert_eq!(array.index(1), None);
    }

    #[test]
    #[should_panic(expected = "should be < len")]
    fn index_out_of_bounds() {
        let array = [(); 1].iter().copied().collect::<NullArray>();
        array.index_checked(1);
    }

    #[test]
    fn into_iter_nullable() {
        let input = [Some(()), None, Some(()), None];
        let array = input.iter().copied().collect::<NullArray<_, Nullable>>();
        assert_eq!(array.is_valid(0), Some(true));
        assert_eq!(array.is_null(1), Some(true));
        assert_eq!(array.is_valid(2), Some(true));
        assert_eq!(array.is_valid(3), Some(false));
        assert_eq!(array.is_valid(4), None);
        assert_eq!(input, array.into_iter().collect::<Vec<_>>().as_slice());
    }

    #[test]
    fn size_of() {
        assert_eq!(mem::size_of::<NullArray<()>>(), mem::size_of::<usize>());
        assert_eq!(
            mem::size_of::<NullArray<(), Nullable>>(),
            mem::size_of::<NullArray<()>>() + mem::size_of::<Bitmap>()
        );
    }
}
