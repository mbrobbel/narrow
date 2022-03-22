//! Validity of array data.

use crate::{Buffer, DataBuffer, Length, Null, Nullable, DEFAULT_ALIGNMENT};
use std::{hint::unreachable_unchecked, ops::Deref};

/// Variants for nullable and non-nullable data.
///
/// The const generic `N` indicates the nullability of the wrapped data.
/// `Validity<_, true>` allocates a validity bitmap that is used to store
/// locations of non-valid (null) values in the buffer. `Validity<_ false>`
/// skips allocation of the validity bitmap.
///
/// The variants in this enum should only be constructed with the following
/// configuration (the const generic N encodes the discriminant):
///
/// - [Validity::Nullable] when `N` is [true].
/// - [Validity::Valid] when `N` is [false].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum RawValidity<T, const N: bool, const A: usize = DEFAULT_ALIGNMENT> {
    Nullable(Nullable<T, A>),
    Valid(T),
}

/// Validity wrapper for nullable and non-nullable data.
///
/// Wraps `T` with validity information (in a [Nullable]) when `N` is [true].
/// Wraps `T` directly when `N` is [false].
///
/// Validity implements [Deref] to get access to the wrapped data.
///
/// - When `N` is [true] (nullable data) the deref target is a Nullable.
/// - When `N` is [false] (non-nullable data) the deref target is `T`.
// Can't control enum variant visibility so wrapping RawValidity to prevent
// users from breaking the const generic discriminant encoding contract.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Validity<T, const N: bool, const A: usize = DEFAULT_ALIGNMENT>(RawValidity<T, N, A>);

impl<T, const N: bool, const A: usize, const B: usize> DataBuffer<T, B>
    for Validity<Buffer<T, B>, N, A>
{
    fn data_buffer(&self) -> &Buffer<T, B> {
        match &self.0 {
            RawValidity::Valid(data) => data,
            RawValidity::Nullable(nullable) => nullable.data_buffer(),
        }
    }
}

impl<T, const A: usize> Deref for Validity<T, false, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            RawValidity::Valid(data) => data,
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl<T, const A: usize> Deref for Validity<T, true, A> {
    type Target = Nullable<T, A>;

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            RawValidity::Nullable(nullable) => nullable,
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl<T, const A: usize> From<Nullable<T, A>> for Validity<T, true, A> {
    fn from(nullable: Nullable<T, A>) -> Self {
        Self(RawValidity::Nullable(nullable))
    }
}

impl<T, const N: bool, const A: usize> Length for Validity<T, N, A>
where
    Self: Deref,
    <Self as Deref>::Target: Length,
{
    fn len(&self) -> usize {
        self.deref().len()
    }
}

impl<T, const A: usize> Null for Validity<T, true, A>
where
    T: Length,
{
    unsafe fn is_valid_unchecked(&self, index: usize) -> bool {
        self.deref().is_valid_unchecked(index)
    }

    fn valid_count(&self) -> usize {
        self.deref().valid_count()
    }

    fn null_count(&self) -> usize {
        self.deref().null_count()
    }
}

impl<T, const A: usize> Null for Validity<T, false, A>
where
    T: Length,
{
    unsafe fn is_valid_unchecked(&self, _index: usize) -> bool {
        true
    }

    fn valid_count(&self) -> usize {
        self.len()
    }

    // todo(mb): make sure compiler optimizes other fns
}

impl<T, U, const A: usize> FromIterator<U> for Validity<T, false, A>
where
    T: FromIterator<U>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = U>,
    {
        // Safety:
        // - Valid: `N` is `false` so it doesn't break the const generic
        //   discriminant encoding.
        Self(RawValidity::Valid(iter.into_iter().collect()))
    }
}

impl<T, U, const A: usize> FromIterator<Option<U>> for Validity<T, true, A>
where
    T: FromIterator<U>,
    U: Default,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<U>>,
    {
        // Safety:
        // - Nullable: `N` is `true` so it doesn't break the const generic
        //   discriminant encoding.
        Self(RawValidity::Nullable(iter.into_iter().collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bitmap, Buffer};

    #[test]
    fn data() {
        let valid: Validity<Buffer<_>, false> = vec![1u8, 2, 3, 4].into_iter().collect();
        assert_eq!(&valid[..], &[1, 2, 3, 4]);

        let nullable: Validity<Buffer<_>, true> = vec![Some(1u8), None, Some(3), Some(4)]
            .into_iter()
            .collect();
        assert_eq!(
            nullable.into_iter().collect::<Vec<_>>(),
            [Some(&1), None, Some(&3), Some(&4)]
        );
    }

    #[test]
    fn array_data() {
        let valid: Validity<Buffer<_>, false> = vec![1u8, 2, 3, 4].into_iter().collect();
        assert_eq!(valid.len(), 4);
        assert_eq!(valid.is_null(1), Some(false));
        assert_eq!(valid.null_count(), 0);
        assert_eq!(valid.is_valid(1), Some(true));
        assert_eq!(valid.valid_count(), 4);

        let nullable: Validity<Buffer<_>, true> = vec![Some(1u8), None, Some(3), Some(4)]
            .into_iter()
            .collect();
        assert_eq!(nullable.len(), 4);
        assert_eq!(nullable.is_null(0), Some(false));
        assert_eq!(nullable.is_null(1), Some(true));
        assert_eq!(nullable.is_null(2), Some(false));
        assert_eq!(nullable.is_null(3), Some(false));
        assert_eq!(nullable.is_null(4), None);
        assert_eq!(nullable.null_count(), 1);
        assert_eq!(nullable.is_valid(0), Some(true));
        assert_eq!(nullable.is_valid(1), Some(false));
        assert_eq!(nullable.is_valid(2), Some(true));
        assert_eq!(nullable.is_valid(3), Some(true));
        assert_eq!(nullable.is_valid(4), None);
        assert_eq!(nullable.valid_count(), 3);
    }

    #[test]
    fn deref() {
        let valid: Validity<Buffer<_>, false> = vec![1u8, 2, 3, 4].into_iter().collect();
        assert_eq!(valid.len(), 4);
        assert_eq!(valid.valid_count(), 4);
        assert_eq!(valid.null_count(), 0);

        let valid: Validity<Bitmap, false> = vec![true, false, true, true].into_iter().collect();
        assert_eq!(valid.len(), 4);
        assert_eq!(valid.valid_count(), 4);
        assert_eq!(valid.null_count(), 0);
        assert_eq!(valid.is_valid(0), Some(true));
        assert_eq!(valid.is_valid(1), Some(true));
        assert_eq!(valid.is_valid(2), Some(true));
        assert_eq!(valid.is_valid(3), Some(true));

        let nullable: Validity<Buffer<_>, true> = vec![Some(1u8), None, Some(3), Some(4)]
            .into_iter()
            .collect();
        assert_eq!(nullable.len(), 4);
        assert_eq!(nullable.valid_count(), 3);
        assert_eq!(nullable.null_count(), 1);
        assert_eq!(
            nullable
                .into_iter()
                .map(|opt| opt.is_some())
                .collect::<Vec<_>>(),
            [true, false, true, true]
        );

        let nullable: Validity<Bitmap, true> = vec![Some(true), Some(false), None, Some(true)]
            .into_iter()
            .collect();
        assert_eq!(nullable.len(), 4);
        assert_eq!(nullable.valid_count(), 3);
        assert_eq!(nullable.null_count(), 1);
        assert_eq!(nullable.is_valid(0), Some(true));
        assert_eq!(nullable.is_valid(1), Some(true));
        // This should select the null impl from the validity bitmap of the
        // validity, not from the data validity.
        assert_eq!(nullable.is_null(2), Some(true));
        assert_eq!(nullable.is_valid(3), Some(true));
    }

    #[test]
    fn into_iter() {
        let valid: Validity<Buffer<_>, false> = vec![1u8, 2, 3, 4].into_iter().collect();
        assert_eq!(valid.into_iter().collect::<Vec<_>>(), vec![&1, &2, &3, &4]);

        let nullable: Validity<Buffer<_>, true> = vec![Some(1u8), None, Some(3), Some(4)]
            .into_iter()
            .collect();
        assert_eq!(
            nullable.into_iter().collect::<Vec<_>>(),
            vec![Some(&1u8), None, Some(&3), Some(&4)]
        );
    }
}
