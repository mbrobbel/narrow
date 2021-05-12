use crate::{Data, Nullable};
use std::{hint::unreachable_unchecked, iter::FromIterator, ops::Deref};

/// Variants for nullable and non-nullable data.
///
/// Wraps data of either a [Buffer](crate::Buffer) or [Bitmap](crate::Bitmap)
/// with validity information.
///
/// The const generic `N` indicates the nullability of the wrapped data.
/// `Validity<_, true>` allocates a validity bitmap that is used to store
/// locations of non-valid (null) values in the buffer. `Validity<_ false>`
/// skips allocation of the validity bitmap.
///
/// The variants in this enum can only be constructed with the following
/// configuration (the const generic N encodes the discriminant):
///
/// - [Validity::Nullable] when `N`=[true]
/// - [Validity::Valid] when `N`=[false]
#[derive(Debug)]
pub enum Validity<T, const N: bool>
where
    T: Data,
{
    Nullable(Nullable<T>),
    Valid(T),
}

impl<T> Deref for Validity<T, false>
where
    T: Data,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Validity::Valid(data) => data,
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl<T> Deref for Validity<T, true>
where
    T: Data,
{
    type Target = Nullable<T>;

    fn deref(&self) -> &Self::Target {
        match self {
            Validity::Nullable(nullable) => nullable,
            // Safety:
            // - The const generic `N` encodes the discriminant.
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl<T, U, const N: usize> From<[U; N]> for Validity<T, false>
where
    T: Data + From<[U; N]>,
{
    fn from(array: [U; N]) -> Self {
        Self::Valid(array.into())
    }
}

impl<T, U> From<Box<[U]>> for Validity<T, false>
where
    T: Data + From<Box<[U]>>,
{
    fn from(boxed_slice: Box<[U]>) -> Self {
        Self::Valid(boxed_slice.into())
    }
}

impl<T, U> From<&[U]> for Validity<T, false>
where
    T: Data,
    for<'a> T: From<&'a [U]>,
{
    fn from(slice: &[U]) -> Self {
        Self::Valid(slice.into())
    }
}

impl<T, U> From<Vec<U>> for Validity<T, false>
where
    T: Data + From<Vec<U>>,
{
    fn from(vec: Vec<U>) -> Self {
        Self::Valid(vec.into())
    }
}

impl<T, U, const N: usize> From<[Option<U>; N]> for Validity<T, true>
where
    T: Data + From<Vec<U>>,
    U: Copy + Default,
{
    fn from(array: [Option<U>; N]) -> Self {
        Self::Nullable(array.iter().copied().collect())
    }
}

impl<T, U> From<Box<[Option<U>]>> for Validity<T, true>
where
    T: Data + From<Vec<U>>,
    U: Copy + Default,
{
    fn from(boxed_slice: Box<[Option<U>]>) -> Self {
        Self::Nullable(boxed_slice.iter().copied().collect())
    }
}

impl<T, U> From<Vec<Option<U>>> for Validity<T, true>
where
    T: Data + From<Vec<U>>,
    U: Copy + Default,
{
    fn from(vec: Vec<Option<U>>) -> Self {
        Self::Nullable(vec.into_iter().collect())
    }
}

impl<T, U> FromIterator<U> for Validity<T, false>
where
    T: Data + FromIterator<U>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = U>,
    {
        Validity::Valid(iter.into_iter().collect())
    }
}

impl<T, U> FromIterator<Option<U>> for Validity<T, true>
where
    T: Data + From<Vec<U>>,
    U: Copy + Default,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<U>>,
    {
        Validity::Nullable(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bitmap, Buffer, Data, ALIGNMENT};

    #[test]
    fn deref() {
        let valid: Validity<Buffer<_, ALIGNMENT>, false> = vec![1u8, 2, 3, 4].into();
        assert_eq!(valid.len(), 4);

        let valid: Validity<Bitmap, false> = vec![true, false, true, true].into();
        assert_eq!(valid.len(), 4);

        let nullable: Validity<Buffer<_, ALIGNMENT>, true> =
            vec![Some(1u8), None, Some(3), Some(4)].into();
        assert_eq!(nullable.len(), 4);

        let valid: Validity<Bitmap, true> = vec![Some(true), Some(false), None, Some(true)].into();
        assert_eq!(valid.len(), 4);
    }

    #[test]
    fn from_array() {
        let array = [1u8, 2, 3, 4];
        let validity: Validity<Buffer<_, ALIGNMENT>, false> = array.into();
        assert_eq!(&validity[..], &array[..]);
        match validity {
            Validity::Nullable(_) => panic!(),
            Validity::Valid(_) => {}
        }

        let array = [Some(1u8), None, Some(3), Some(4)];
        let validity: Validity<Buffer<_, ALIGNMENT>, true> = array.into();
        assert_eq!(validity.into_iter().collect::<Vec<_>>(), array);
        match validity {
            Validity::Nullable(_) => {}
            Validity::Valid(_) => panic!(),
        }

        let array = [true, false, true, false];
        let validity: Validity<Bitmap, false> = array.into();
        assert_eq!(validity.into_iter().collect::<Vec<_>>(), &array[..]);
        match validity {
            Validity::Nullable(_) => panic!(),
            Validity::Valid(_) => {}
        }

        let array = [Some(false), None, Some(true), Some(true)];
        let validity: Validity<Bitmap, true> = array.into();
        assert_eq!(validity.into_iter().collect::<Vec<_>>(), array);
        match validity {
            Validity::Nullable(_) => {}
            Validity::Valid(_) => panic!(),
        }
    }

    #[test]
    fn from_iter() {
        let vec = vec![1u8, 2, 3, 4];
        let validity: Validity<Buffer<_, ALIGNMENT>, false> = vec.clone().into_iter().collect();
        assert_eq!(&validity[..], &vec[..]);

        let vec = vec![Some(1u8), None, Some(3), Some(4)];
        let validity: Validity<Buffer<_, ALIGNMENT>, true> = vec.clone().into_iter().collect();
        assert_eq!(validity.into_iter().collect::<Vec<_>>(), vec);
    }
}
