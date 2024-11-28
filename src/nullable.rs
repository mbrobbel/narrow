//! Nullable data.

use crate::{
    bitmap::{Bitmap, BitmapIntoIter, BitmapIter, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{self, BufferMut, BufferRef, BufferRefMut, BufferType, VecBuffer},
    FixedSize, Index, Length,
};
use std::{
    borrow::Borrow,
    fmt::{Debug, Formatter, Result},
    iter::{Map, Zip},
};

/// Wrapper for nullable data.
///
/// Store data with a validity [Bitmap] that uses a single bit per value in `T`
/// that indicates the validity (non-nullness) or invalidity (nullness) of that value.
pub struct Nullable<T, Buffer: BufferType = VecBuffer> {
    /// Data that may contain null elements.
    pub(crate) data: T,

    /// The validity bitmap with validity information for the elements in the
    /// data.
    pub(crate) validity: Bitmap<Buffer>,
}

impl<T: Length, Buffer: BufferType> From<T> for Nullable<T, Buffer>
where
    Bitmap<Buffer>: FromIterator<bool>,
{
    fn from(data: T) -> Self {
        let validity = Bitmap::new_valid(data.len());
        Self { data, validity }
    }
}

impl<T, Buffer: BufferType> AsRef<T> for Nullable<T, Buffer> {
    fn as_ref(&self) -> &T {
        &self.data
    }
}

impl<T, Buffer: BufferType> BitmapRef for Nullable<T, Buffer> {
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        &self.validity
    }
}

impl<T, Buffer: BufferType> BitmapRefMut for Nullable<T, Buffer> {
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        &mut self.validity
    }
}

impl<T, U, Buffer: BufferType> BufferRef<U> for Nullable<T, Buffer>
where
    U: FixedSize,
    T: buffer::Buffer<U>,
{
    type Buffer = T;

    fn buffer_ref(&self) -> &Self::Buffer {
        &self.data
    }
}

impl<T, U, Buffer: BufferType> BufferRefMut<U> for Nullable<T, Buffer>
where
    U: FixedSize,
    T: BufferMut<U>,
{
    type BufferMut = T;

    fn buffer_ref_mut(&mut self) -> &mut Self::BufferMut {
        &mut self.data
    }
}

impl<T: Debug, Buffer: BufferType> Debug for Nullable<T, Buffer> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Nullable")
            .field("data", &self.data)
            .field("validity", &self.validity)
            .finish()
    }
}

impl<T: Clone, Buffer: BufferType> Clone for Nullable<T, Buffer>
where
    Bitmap<Buffer>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            validity: self.validity.clone(),
        }
    }
}

impl<T: Default, Buffer: BufferType> Default for Nullable<T, Buffer>
where
    Bitmap<Buffer>: Default,
{
    fn default() -> Self {
        Self {
            data: T::default(),
            validity: Bitmap::default(),
        }
    }
}

impl<T: Extend<U>, U: Default, V: Borrow<bool>, Buffer: BufferType> Extend<(V, U)>
    for Nullable<T, Buffer>
where
    <Buffer as BufferType>::Buffer<u8>: BufferMut<u8> + Extend<u8>,
{
    fn extend<I: IntoIterator<Item = (V, U)>>(&mut self, iter: I) {
        // https://github.com/rust-lang/rust-clippy/issues/9378
        #[allow(clippy::pattern_type_mismatch)]
        self.data.extend(
            iter.into_iter()
                .inspect(|(valid, _value)| {
                    self.validity.extend(std::iter::once(valid.borrow()));
                })
                .map(|(_, item)| item),
        );
    }
}

impl<T: Extend<U>, U: Default, Buffer: BufferType> Extend<Option<U>> for Nullable<T, Buffer>
where
    <Buffer as BufferType>::Buffer<u8>: BufferMut<u8> + Extend<u8>,
{
    fn extend<I: IntoIterator<Item = Option<U>>>(&mut self, iter: I) {
        self.extend(
            iter.into_iter()
                .map(|opt| (opt.is_some(), opt.unwrap_or_default())),
        );
    }
}

impl<'a, T, U, Buffer: BufferType> FromIterator<&'a Option<U>> for Nullable<T, Buffer>
where
    T: Default + Extend<U>,
    U: Copy + Default,
    <Buffer as BufferType>::Buffer<u8>: BufferMut<u8> + Default + Extend<u8>,
{
    fn from_iter<I: IntoIterator<Item = &'a Option<U>>>(iter: I) -> Self {
        let (validity, data) = iter
            .into_iter()
            .map(|opt| (opt.is_some(), opt.as_ref().copied().unwrap_or_default()))
            .unzip();
        Self { data, validity }
    }
}

impl<T, U, Buffer: BufferType> FromIterator<Option<U>> for Nullable<T, Buffer>
where
    T: Default + Extend<U>,
    U: Default,
    <Buffer as BufferType>::Buffer<u8>: BufferMut<u8> + Default + Extend<u8>,
{
    fn from_iter<I: IntoIterator<Item = Option<U>>>(iter: I) -> Self {
        let (validity, data) = iter
            .into_iter()
            .map(|opt| (opt.is_some(), opt.unwrap_or_default()))
            .unzip();
        Self { data, validity }
    }
}

impl<T, Buffer: BufferType> Index for Nullable<T, Buffer>
where
    T: Index,
{
    type Item<'a>
        = Option<<T as Index>::Item<'a>>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.is_valid_unchecked(index)
            .then(|| self.data.index_unchecked(index))
    }
}

impl<'a, T, Buffer: BufferType> IntoIterator for &'a Nullable<T, Buffer>
where
    &'a T: IntoIterator,
{
    type Item = Option<<&'a T as IntoIterator>::Item>;
    type IntoIter = Map<
        Zip<BitmapIter<'a>, <&'a T as IntoIterator>::IntoIter>,
        fn((bool, <&'a T as IntoIterator>::Item)) -> Self::Item,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.validity
            .into_iter()
            .zip(&self.data)
            .map(|(validity, value)| validity.then_some(value))
    }
}

impl<T, Buffer: BufferType> IntoIterator for Nullable<T, Buffer>
where
    T: IntoIterator,
    <Buffer as BufferType>::Buffer<u8>: IntoIterator<Item = u8>,
{
    type Item = Option<<T as IntoIterator>::Item>;
    type IntoIter = Map<
        Zip<
            BitmapIntoIter<<<Buffer as BufferType>::Buffer<u8> as IntoIterator>::IntoIter>,
            <T as IntoIterator>::IntoIter,
        >,
        fn((bool, <T as IntoIterator>::Item)) -> Self::Item,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.validity
            .into_iter()
            .zip(self.data)
            .map(|(validity, value)| validity.then_some(value))
    }
}

impl<T, Buffer: BufferType> Length for Nullable<T, Buffer> {
    fn len(&self) -> usize {
        self.validity.len()
    }
}

impl<T, Buffer: BufferType> PartialEq for Nullable<T, Buffer>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.data == other.data && self.validity == other.validity
    }
}

impl<T: IntoIterator<Item = U>, U: PartialEq, Buffer: BufferType> PartialEq<[Option<U>]>
    for Nullable<T, Buffer>
where
    for<'a> &'a Self: IntoIterator<Item = Option<U>>,
{
    fn eq(&self, other: &[Option<U>]) -> bool {
        self.len() == other.len() && self.into_iter().zip(other).all(|(a, b)| &a == b)
    }
}

impl<T, Buffer: BufferType> ValidityBitmap for Nullable<T, Buffer> {}

#[cfg(test)]
mod tests {

    use super::*;
    use std::{
        iter::{self, Repeat, Take},
        mem,
    };

    #[test]
    fn from_iter() {
        let input = [Some(1), Some(2), Some(3), Some(4), None, Some(42)];
        let nullable = input.into_iter().collect::<Nullable<Vec<_>>>();
        assert_eq!(nullable.buffer_ref(), &[1, 2, 3, 4, u32::default(), 42]);
        assert_eq!(nullable.bitmap_ref().buffer_ref(), &[0b0010_1111]);
        assert_eq!(
            (&nullable)
                .into_iter()
                .map(Option::<&_>::copied)
                .collect::<Vec<_>>(),
            input
        );
        assert_eq!(nullable.len(), 6);
    }

    #[test]
    fn from_iter_array() {
        let input = [Some([1234, 1234]), None, Some([42, 42])];
        let mut nullable = input.into_iter().collect::<Nullable<Vec<_>>>();
        assert_eq!(
            <_ as BufferRef<u32>>::buffer_ref(&nullable).as_slice(),
            &[[1234, 1234], [u32::default(), u32::default()], [42, 42]]
        );
        assert_eq!(nullable.bitmap_ref().buffer_ref(), &[0b00101]);
        <_ as BufferRefMut<u32>>::buffer_ref_mut(&mut nullable).as_mut_slice()[0] = [4321, 4321];
        assert_eq!(
            <_ as BufferRef<u32>>::buffer_ref(&nullable).as_slice(),
            &[[4321, 4321], [u32::default(), u32::default()], [42, 42]]
        );
        assert_eq!(<_ as BufferRef<u32>>::buffer_ref(&nullable).len(), 3);
        assert_eq!(nullable.len(), 3);
        nullable.bitmap_ref_mut().buffer_ref_mut()[0] = 0b00111;
        assert_eq!(
            nullable.into_iter().collect::<Vec<_>>(),
            [Some([4321, 4321]), Some([0, 0]), Some([42, 42])]
        );
    }

    #[test]
    fn into_iter() {
        let input = [Some(1), Some(2), Some(3), Some(4), None, Some(42)];
        let nullable = input.into_iter().collect::<Nullable<Vec<_>>>();
        let output = nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());
    }

    #[test]
    fn index() {
        let input = [Some(1), Some(2), None, Some(4)];
        let nullable = input.into_iter().collect::<Nullable<Vec<_>>>();
        assert_eq!(nullable.index(0), Some(Some(&1)));
        assert_eq!(nullable.index_checked(0), Some(&1));
        assert_eq!(nullable.index(1), Some(Some(&2)));
        assert_eq!(nullable.index_checked(1), Some(&2));
        assert_eq!(nullable.index(2), Some(None));
        assert_eq!(nullable.index_checked(2), None);
        assert_eq!(nullable.index(3), Some(Some(&4)));
        assert_eq!(nullable.index_checked(3), Some(&4));
        assert_eq!(nullable.index(4), None);
    }

    #[test]
    #[should_panic(expected = "should be < len")]
    fn index_checked() {
        let input = [Some(1), None];
        let nullable = input.into_iter().collect::<Nullable<Vec<_>>>();
        let _ = nullable.index_checked(2);
    }

    #[test]
    fn opt_bool_iter() {
        let input = [Some(true), Some(false), None];
        let nullable = input.into_iter().collect::<Nullable<Bitmap>>();
        assert_eq!(nullable.as_ref().buffer_ref(), &[0b0000_0001]);
        assert_eq!(nullable.bitmap_ref().buffer_ref(), &[0b0000_0011]);
    }

    #[test]
    fn count_iter() {
        #[derive(Default)]
        struct Count(usize);

        impl<T> FromIterator<T> for Count {
            fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
                Self(iter.into_iter().count())
            }
        }

        impl<T> Extend<T> for Count {
            fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
                self.0 += iter.into_iter().count();
            }
        }

        impl IntoIterator for Count {
            type IntoIter = Take<Repeat<()>>;
            type Item = ();

            fn into_iter(self) -> Self::IntoIter {
                iter::repeat(()).take(self.0)
            }
        }

        let input = [Some(()), Some(()), None];
        let nullable = input.into_iter().collect::<Nullable<Count>>();
        assert_eq!(nullable.bitmap_ref().buffer_ref(), &[0b0000_0011]);
        assert_eq!(nullable.into_iter().collect::<Vec<Option<()>>>(), input);
    }

    #[test]
    fn size_of() {
        assert_eq!(mem::size_of::<Nullable<()>>(), mem::size_of::<Bitmap>());
    }
}
