//! Validity wrapper for nullable and non-nullable data.

use std::{hint::unreachable_unchecked, ops::Deref};

use crate::{
    buffer::{Buffer, BufferAlloc, BufferExtend},
    nullable::Nullable,
    DataBuffer, Primitive,
};

/// Variants for nullable and non-nullable data of type `T`.
#[derive(Debug)]
enum RawValidity<T, U = Vec<u8>>
where
    U: Buffer<u8>,
{
    Nullable(Nullable<T, U>),
    Valid(T),
}

/// Validity wrapper for nullable and non-nullable data.
///
/// The const generic `N` encodes the nullability of the wrapped data.
///
/// - When `N` is [true] the data is nullable and wrapped in a [Nullable].
/// - When `N` is [false] the data is non-nullable.
///
/// Validity implements [Deref] to get access to the wrapped data.
///
/// # Safety
/// - All methods that construct a Validity must ensure the value of `N`
///   correctly encodes the discriminant of the wrapped variant.
// TODO: pub(crate)?
#[derive(Debug)]
pub struct Validity<T, const N: bool, U = Vec<u8>>(RawValidity<T, U>)
where
    U: Buffer<u8>;

// TODO: replace this with a blanket impl for DataBuffer?
impl<T, U, V> DataBuffer<V> for Validity<T, false, U>
where
    T: Buffer<V>,
    U: Buffer<u8>,
    V: Primitive,
{
    type Buffer = T;

    fn data_buffer(&self) -> &Self::Buffer {
        match &self.0 {
            RawValidity::Valid(data) => data,
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl<T, U> Deref for Validity<T, false, U>
where
    U: Buffer<u8>,
{
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

impl<T, U> Deref for Validity<T, true, U>
where
    U: Buffer<u8>,
{
    type Target = Nullable<T, U>;

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            RawValidity::Nullable(nullable) => nullable,
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl<T, U, V> FromIterator<V> for Validity<T, false, U>
where
    T: BufferAlloc<V>,
    U: Buffer<u8>,
    V: Primitive,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = V>,
    {
        // Safety:
        // - Valid: `N` is `false` so it doesn't break the const generic discriminant
        //   encoding.
        Self(RawValidity::Valid(iter.into_iter().collect()))
    }
}

impl<T, U, V> FromIterator<Option<V>> for Validity<T, true, U>
where
    T: Default + BufferExtend<V>,
    U: Default + BufferExtend<u8>,
    V: Primitive,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<V>>,
    {
        // Safety:
        // - Nullable: `N` is `true` so it doesn't break the const generic discriminant
        //   encoding.
        Self(RawValidity::Nullable(iter.into_iter().collect()))
    }
}

impl<T, U> IntoIterator for Validity<T, false, U>
where
    U: Buffer<u8>,
    T: IntoIterator,
{
    type IntoIter = <T as IntoIterator>::IntoIter;
    type Item = <T as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            RawValidity::Valid(data) => data.into_iter(),
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl<T, U> IntoIterator for Validity<T, true, U>
where
    U: Buffer<u8>,
    Nullable<T, U>: IntoIterator,
{
    type IntoIter = <Nullable<T, U> as IntoIterator>::IntoIter;
    type Item = <Nullable<T, U> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            RawValidity::Nullable(nullable) => nullable.into_iter(),
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ValidityBitmap;

    #[test]
    fn from_iter() {
        let nullable = [Some(1u8), None, Some(2)];
        let validity = nullable.into_iter().collect::<Validity<Vec<_>, true>>();
        assert_eq!(validity.data_buffer(), &[1, u8::default(), 2]);
        assert_eq!(
            validity
                .validity_bitmap()
                .into_iter()
                .collect::<Vec<bool>>(),
            [true, false, true]
        );

        let non_nullable = [1u8, 2, 3, 4];
        let validity = non_nullable
            .into_iter()
            .collect::<Validity<Vec<_>, false>>();
        assert_eq!(validity.data_buffer(), &[1, 2, 3, 4]);
    }

    #[test]
    fn deref_into_iter() {
        let nullable = vec![Some(1u8), None, Some(2)];
        let validity = nullable
            .clone()
            .into_iter()
            .collect::<Validity<Vec<_>, true>>();
        assert_eq!(nullable, validity.into_iter().collect::<Vec<_>>());

        let non_nullable = vec![1u8, 2, 3, 4];
        let validity = non_nullable
            .clone()
            .into_iter()
            .collect::<Validity<Vec<_>, false>>();
        assert_eq!(non_nullable, validity.into_iter().collect::<Vec<u8>>());
    }
}
