use crate::{ArrayData, Bitmap, BitmapIter, Buffer, Primitive, Validity, ALIGNMENT};
use bitvec::{order::Lsb0, slice::BitValIter};
use std::{
    iter::{self, Copied, Zip},
    num::TryFromIntError,
    ops::Deref,
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
#[derive(Clone, Debug)]
pub struct Offset<T, const N: bool>(Validity<Buffer<T, ALIGNMENT>, N>)
where
    T: OffsetValue;

// Non-nullable validity deref gives the offset buffer, which
// always has one more value than the array.
impl<T, const N: bool> ArrayData for Offset<T, N>
where
    T: OffsetValue,
{
    fn len(&self) -> usize {
        match N {
            false => self.0.len() - 1,
            true => self.0.len(),
        }
    }

    fn is_null(&self, index: usize) -> bool {
        self.0.is_null(match N {
            false => index + 1,
            true => index,
        })
    }

    fn null_count(&self) -> usize {
        match N {
            false => 0,
            true => self.0.null_count(),
        }
    }

    fn is_valid(&self, index: usize) -> bool {
        self.0.is_valid(match N {
            false => index + 1,
            true => index,
        })
    }

    fn valid_count(&self) -> usize {
        match N {
            false => self.0.len() - 1,
            true => self.0.valid_count(),
        }
    }
}

impl<T> Deref for Offset<T, false>
where
    T: OffsetValue,
{
    type Target = <Validity<Buffer<T, ALIGNMENT>, false> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Deref for Offset<T, true>
where
    T: OffsetValue,
{
    type Target = <Validity<Buffer<T, ALIGNMENT>, true> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        &self.0
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

        Self(Validity::nullable(validity, buffer.into_iter().collect()))
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

impl<'a, T> Iterator for OffsetIter<Zip<BitValIter<'a, Lsb0, u8>, Copied<Iter<'a, T>>>, true>
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

    #[test]
    fn array_data() {
        let offset: Offset<i64, false> = [1, 2, 3, 4].into_iter().collect();
        assert_eq!(offset.len(), 4);
        assert!(!offset.is_null(0));
        assert_eq!(offset.null_count(), 0);
        assert!(offset.is_valid(0));
        assert_eq!(offset.valid_count(), 4);

        let offset: Offset<i32, true> = [Some(3), None, Some(4), Some(0)].into_iter().collect();
        assert_eq!(offset.len(), 4);
        assert!(!offset.is_null(0));
        assert!(offset.is_null(1));
        assert_eq!(offset.null_count(), 1);
        assert!(offset.is_valid(0));
        assert!(!offset.is_valid(1));
        assert_eq!(offset.valid_count(), 3);
    }

    #[test]
    fn from_iter() {
        let offset: Offset<i64, false> = [1, 2, 3, 4].into_iter().collect();
        assert_eq!(&offset[..], &[0, 1, 3, 6, 10]);

        let offset: Offset<i32, true> = [Some(3), None, Some(4), Some(0)].into_iter().collect();
        assert_eq!(&offset.data()[..], &[0, 3, 3, 7, 7]);
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
