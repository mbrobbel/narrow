use crate::{ArrayData, Bitmap, Nullable};
use std::{hint::unreachable_unchecked, iter::FromIterator, ops::Deref};

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
#[derive(Clone, Debug)]
enum RawValidity<T, const N: bool> {
    Nullable(Nullable<T>),
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
#[derive(Clone, Debug)]
pub struct Validity<T, const N: bool>(RawValidity<T, N>);

impl<T, const N: bool> Validity<T, N> {
    /// Returns a reference to the data wrapped by this [Validity].
    /// This uses the deref impl to either get the data from the [Nullable] or
    /// directly in the case of non-nullable data.
    pub(crate) fn data(&self) -> &T {
        match (N, &self.0) {
            (false, RawValidity::Valid(data)) => data,
            (true, RawValidity::Nullable(nullable)) => nullable.data(),
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl<T> Validity<T, true> {
    /// Returns a new Validity with nullable data.
    pub(crate) fn nullable(validity: Bitmap, data: T) -> Self {
        // Safety:
        // - Nullable: `N` is `true` so it doesn't break the const generic
        //   discriminant encoding.
        Self(RawValidity::Nullable(Nullable::new(data, validity)))
    }
}

impl<T, const N: bool> ArrayData for Validity<T, N>
where
    T: ArrayData,
{
    fn len(&self) -> usize {
        match (N, &self.0) {
            (false, RawValidity::Valid(data)) => data.len(),
            (true, RawValidity::Nullable(nullable)) => nullable.len(),
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
    fn is_null(&self, index: usize) -> bool {
        match (N, &self.0) {
            (false, _) => false,
            (true, RawValidity::Nullable(nullable)) => nullable.is_null(index),
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
    fn null_count(&self) -> usize {
        match (N, &self.0) {
            (false, _) => 0,
            (true, RawValidity::Nullable(nullable)) => nullable.null_count(),
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
    fn is_valid(&self, index: usize) -> bool {
        match (N, &self.0) {
            (false, _) => true,
            (true, RawValidity::Nullable(nullable)) => nullable.is_valid(index),
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
    fn valid_count(&self) -> usize {
        match (N, &self.0) {
            (false, RawValidity::Valid(data)) => data.len(),
            (true, RawValidity::Nullable(nullable)) => nullable.valid_count(),
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl<T> Deref for Validity<T, false> {
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

impl<T> Deref for Validity<T, true> {
    type Target = Nullable<T>;

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            RawValidity::Nullable(nullable) => nullable,
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl<T, U> FromIterator<U> for Validity<T, false>
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

impl<T, U> FromIterator<Option<U>> for Validity<T, true>
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
    use crate::{Bitmap, Buffer, ALIGNMENT};

    #[test]
    fn data() {
        let valid: Validity<Buffer<_, ALIGNMENT>, false> = vec![1u8, 2, 3, 4].into_iter().collect();
        assert_eq!(valid.data().as_slice(), &[1, 2, 3, 4]);

        let nullable: Validity<Buffer<_, ALIGNMENT>, true> =
            vec![Some(1u8), None, Some(3), Some(4)]
                .into_iter()
                .collect();
        assert_eq!(nullable.data().as_slice(), &[1, u8::default(), 3, 4]);
    }

    #[test]
    fn array_data() {
        let valid: Validity<Buffer<_, ALIGNMENT>, false> = vec![1u8, 2, 3, 4].into_iter().collect();
        assert_eq!(valid.len(), 4);
        assert!(!valid.is_null(1));
        assert_eq!(valid.null_count(), 0);
        assert!(valid.is_valid(1));
        assert_eq!(valid.valid_count(), 4);

        let nullable: Validity<Buffer<_, ALIGNMENT>, true> =
            vec![Some(1u8), None, Some(3), Some(4)]
                .into_iter()
                .collect();
        assert_eq!(nullable.len(), 4);
        assert!(!nullable.is_null(0));
        assert!(nullable.is_null(1));
        assert_eq!(nullable.null_count(), 1);
        assert!(nullable.is_valid(0));
        assert!(!nullable.is_valid(1));
        assert_eq!(nullable.valid_count(), 3);
    }

    #[test]
    fn deref() {
        let valid: Validity<Buffer<_, ALIGNMENT>, false> = vec![1u8, 2, 3, 4].into_iter().collect();
        assert_eq!(valid.len(), 4);

        let valid: Validity<Bitmap, false> = vec![true, false, true, true].into_iter().collect();
        assert_eq!(valid.len(), 4);

        let nullable: Validity<Buffer<_, ALIGNMENT>, true> =
            vec![Some(1u8), None, Some(3), Some(4)]
                .into_iter()
                .collect();
        assert_eq!(nullable.len(), 4);
        assert_eq!(
            nullable.validity(),
            &[true, false, true, true]
                .iter()
                .copied()
                .collect::<Bitmap>()
        );

        let valid: Validity<Bitmap, true> = vec![Some(true), Some(false), None, Some(true)]
            .into_iter()
            .collect();
        assert_eq!(valid.len(), 4);
    }
}
