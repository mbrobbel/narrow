use crate::{Bitmap, BitmapIter, Buffer, Length, Null, Nullable, Primitive, Validity, ALIGN};
use std::{
    iter::{self, Copied, Zip},
    num::TryFromIntError,
    ops::{AddAssign, Deref},
    slice::Iter,
};

/// Types representing offset values.
///
/// Values with these types can be used to represent offset values in an
/// [Offset].
///
/// This trait is sealed to prevent downstream implementations.
pub trait OffsetValue:
    Primitive
    + AddAssign
    + TryFrom<usize, Error = TryFromIntError>
    + TryInto<usize, Error = TryFromIntError>
    + sealed::Sealed
{
}

mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T where T: super::OffsetValue {}
}

impl OffsetValue for i32 {}
impl OffsetValue for i64 {}

/// Wrapper for offset values of variable sized arrays.
///
/// This is a wrapper around a [Validity] with a [Buffer] that provides a
/// different [FromIterator] implementation. In the offset buffer the previous
/// offset is copied for null values, whereas the behavior of [Validity] adds a
/// [Default::default] value.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Offset<T, const N: bool>(Validity<Buffer<T, ALIGN>, N>);

impl<T, const N: bool> Clone for Offset<T, N>
where
    T: Primitive, // todo(mb): trait bound on Clone of Buffer
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for Offset<T, false> {
    // Deref to Validity instead of Buffer directly to get Null impl of Validity
    // for valid data.
    type Target = Validity<Buffer<T, ALIGN>, false>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Deref for Offset<T, true> {
    // Deref to Buffer with data instead of validity bitmap. This makes sure the
    // length information is correct.
    // The Null impl for Offset<T, true> handles index offsets.
    type Target = Buffer<T, ALIGN>;

    fn deref(&self) -> &Self::Target {
        self.0.data()
    }
}

impl<T> Length for Offset<T, true> {
    fn len(&self) -> usize {
        // Deref to buffer with values to get correct length (+1).
        self.deref().len()
    }
}

// Deref of Offset targets the Buffer with values so this impl makes sure we use
// the information from the Nullable for the Null functions.
impl<T> Null for Offset<T, true> {
    fn is_valid(&self, index: usize) -> Option<bool> {
        self.0.deref().is_valid(index)
    }

    unsafe fn is_valid_unchecked(&self, index: usize) -> bool {
        self.0.deref().is_valid_unchecked(index)
    }

    fn valid_count(&self) -> usize {
        self.0.deref().valid_count()
    }

    fn null_count(&self) -> usize {
        self.0.deref().null_count()
    }
}

impl<T> FromIterator<T> for Offset<T, false>
where
    T: OffsetValue,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self(
            iter::once(T::default())
                .chain(iter.into_iter().scan(T::default(), |last, value| {
                    *last += value;
                    Some(*last)
                }))
                .collect(),
        )
    }
}

impl<T> FromIterator<Option<T>> for Offset<T, true>
where
    T: OffsetValue,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<T>>,
    {
        let iter = iter.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();
        let mut buffer = Vec::with_capacity(upper_bound.unwrap_or(lower_bound) + 1);

        buffer.push(T::default());

        let validity = iter
            .scan(T::default(), |last, opt| {
                // Instead of T::default(), we push the previous offset.
                *last += opt.unwrap_or_default();
                buffer.push(*last);
                Some(opt.is_some())
            })
            .collect::<Bitmap>();

        Self(unsafe { Nullable::from_raw_parts(buffer.into_iter().collect(), validity) }.into())
    }
}

/// Iterator over offsets values of variable sized arrays.
pub struct OffsetIter<T, const N: bool> {
    pos: Option<usize>,
    iter: T,
}

impl<'a, T> Iterator for OffsetIter<Copied<Iter<'a, T>>, false>
where
    T: OffsetValue,
{
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // Empty offset array?
        self.pos.and(
            self.iter
                .next()
                .map(|offset| offset.try_into().unwrap())
                .map(|end| {
                    let result = (self.pos.unwrap(), end);
                    self.pos = Some(end);
                    result
                }),
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> Iterator for OffsetIter<Zip<BitmapIter<'a>, Copied<Iter<'a, T>>>, true>
where
    T: OffsetValue,
{
    type Item = Option<(usize, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.and(
            self.iter
                .next()
                .map(|(validity, offset)| (validity, offset.try_into().unwrap()))
                .map(|(validity, end)| {
                    let result = (self.pos.unwrap(), end);
                    self.pos = Some(end);
                    match validity {
                        true => Some(result),
                        false => None,
                    }
                }),
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> IntoIterator for &'a Offset<T, false>
where
    T: OffsetValue,
{
    type Item = (usize, usize);
    type IntoIter = OffsetIter<Copied<Iter<'a, T>>, false>;

    fn into_iter(self) -> Self::IntoIter {
        let mut iter = self.0.into_iter();
        OffsetIter {
            pos: iter.next().map(|offset| offset.try_into().unwrap()),
            iter,
        }
    }
}

impl<'a, T> IntoIterator for &'a Offset<T, true>
where
    T: OffsetValue,
{
    type Item = Option<(usize, usize)>;
    type IntoIter = OffsetIter<Zip<BitmapIter<'a>, Copied<Iter<'a, T>>>, true>;

    fn into_iter(self) -> Self::IntoIter {
        let validity = self.0.validity().into_iter();
        let mut offsets = self.0.data().into_iter();
        let pos = offsets.next().map(|offset| offset.try_into().unwrap());
        let iter = validity.zip(offsets);
        OffsetIter { pos, iter }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Null;

    #[test]
    fn array_data() {
        let offset: Offset<i64, false> = [1, 2, 3, 4].into_iter().collect();
        assert_eq!(offset.len(), 5); // 5 is correct here because the offset stores an additional value
        assert_eq!(offset.is_null(0), Some(false));
        assert_eq!(offset.null_count(), 0);
        assert_eq!(offset.is_valid(0), Some(true));
        assert_eq!(offset.valid_count(), 5);

        let offset: Offset<i32, true> = [Some(3), None, Some(4), Some(0)].into_iter().collect();
        assert_eq!(offset.len(), 5);
        assert_eq!(offset.is_null(0), Some(false));
        assert_eq!(offset.is_null(1), Some(true));
        assert_eq!(offset.is_null(2), Some(false));
        assert_eq!(offset.is_null(3), Some(false));
        assert_eq!(offset.is_null(4), None);
        assert_eq!(offset.null_count(), 1);
        assert_eq!(offset.is_valid(0), Some(true));
        assert_eq!(offset.is_valid(1), Some(false));
        assert_eq!(offset.valid_count(), 3);
    }

    #[test]
    fn from_iter() {
        let offset: Offset<i64, false> = [1, 2, 3, 4].into_iter().collect();
        assert_eq!(&offset[..], &[0, 1, 3, 6, 10]);

        let offset: Offset<i32, true> = [Some(3), None, Some(4), Some(0)].into_iter().collect();
        assert_eq!(&offset[..], &[0, 3, 3, 7, 7]);
    }

    #[test]
    fn into_iter() {
        let offset: Offset<i64, false> = [1, 2, 3, 4].into_iter().collect();
        let offsets = offset.into_iter().collect::<Vec<_>>();
        assert_eq!(offsets, vec![(0, 1), (1, 3), (3, 6), (6, 10)]);

        let offset: Offset<i32, true> = [Some(3), None, Some(4), None, Some(0)]
            .into_iter()
            .collect();
        let offsets = offset.into_iter().collect::<Vec<_>>();
        assert_eq!(
            offsets,
            vec![Some((0, 3)), None, Some((3, 7)), None, Some((7, 7))]
        );
    }
}
