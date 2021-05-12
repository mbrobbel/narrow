use crate::{Bitmap, Buffer, Nullable, OffsetType, Validity, ALIGNMENT};
use std::{
    iter::{self, FromIterator},
    ops::Deref,
};

/// Wrapper for offset values of variable sized arrays.
///
/// This is a wrapper around a [Validity] with a [Buffer] that provides a
/// different [FromIterator] implementation. In the offset buffer the previous
/// offset is copied for null values, whereas the behavior of [Validity] adds a
/// [Default::default] value.
#[derive(Debug)]
pub struct Offset<T, const N: bool>(Validity<Buffer<T, ALIGNMENT>, N>)
where
    T: OffsetType;

impl<T, const N: bool> Deref for Offset<T, N>
where
    T: OffsetType,
{
    type Target = Validity<Buffer<T, ALIGNMENT>, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> From<[T; N]> for Offset<T, false>
where
    T: OffsetType,
{
    fn from(array: [T; N]) -> Self {
        array.iter().copied().collect()
    }
}

impl<T> From<Box<[T]>> for Offset<T, false>
where
    T: OffsetType,
{
    fn from(boxed_slice: Box<[T]>) -> Self {
        boxed_slice.iter().copied().collect()
    }
}

impl<T> From<&[T]> for Offset<T, false>
where
    T: OffsetType,
{
    fn from(slice: &[T]) -> Self {
        slice.iter().copied().collect()
    }
}

impl<T> From<Vec<T>> for Offset<T, false>
where
    T: OffsetType,
{
    fn from(vec: Vec<T>) -> Self {
        vec.into_iter().collect()
    }
}

impl<T, const N: usize> From<[Option<T>; N]> for Offset<T, true>
where
    T: OffsetType,
{
    fn from(array: [Option<T>; N]) -> Self {
        array.iter().copied().collect()
    }
}

impl<T> From<Box<[Option<T>]>> for Offset<T, true>
where
    T: OffsetType,
{
    fn from(boxed_slice: Box<[Option<T>]>) -> Self {
        boxed_slice.iter().copied().collect()
    }
}

impl<T> From<&[Option<T>]> for Offset<T, true>
where
    T: OffsetType,
{
    fn from(slice: &[Option<T>]) -> Self {
        slice.iter().copied().collect()
    }
}

impl<T> From<Vec<Option<T>>> for Offset<T, true>
where
    T: OffsetType,
{
    fn from(vec: Vec<Option<T>>) -> Self {
        vec.into_iter().collect()
    }
}

impl<T> FromIterator<T> for Offset<T, false>
where
    T: OffsetType,
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
    T: OffsetType,
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
                Some(opt)
            })
            .map(|opt| opt.is_some())
            .collect::<Bitmap>();

        Self(Validity::Nullable(Nullable::new(validity, buffer.into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        let offset: Offset<i64, false> = [1, 2, 3, 4].into();
        assert_eq!(&offset[..], &[0, 1, 3, 6, 10]);

        let offset: Offset<i32, true> = [Some(3), None, Some(4), Some(0)].into();
        assert_eq!(offset.data().as_slice(), &[0, 3, 3, 7, 7]);
    }
}
