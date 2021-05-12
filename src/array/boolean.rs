use crate::{Array, Bitmap, Nullable, Validity};
use std::{iter::FromIterator, ops::Deref};

/// Array with boolean values.
///
/// Values are stored using single bits in a [Bitmap].
#[derive(Debug)]
pub struct BooleanArray<const N: bool>(Validity<Bitmap, N>);

impl Array for BooleanArray<false> {
    type Data = Bitmap;

    fn data(&self) -> &Self::Data {
        self
    }
}

impl Array for BooleanArray<true> {
    type Data = Nullable<Bitmap>;

    fn data(&self) -> &Self::Data {
        self
    }
}

impl<const N: bool> Deref for BooleanArray<N> {
    type Target = Validity<Bitmap, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<bool> for BooleanArray<false> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = bool>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<&'a bool> for BooleanArray<false> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a bool>,
    {
        Self(iter.into_iter().collect())
    }
}

impl FromIterator<Option<bool>> for BooleanArray<true> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<bool>>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<&'a Option<bool>> for BooleanArray<true> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a Option<bool>>,
    {
        Self(iter.into_iter().copied().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deref() {
        let vec = vec![true, false, true, false, true, true];
        let boolean_array: BooleanArray<false> = vec.iter().collect();
        assert_eq!(boolean_array.len(), vec.len());
        assert!(&boolean_array[0]);
        assert!(!&boolean_array[1]);
        assert_eq!(vec, boolean_array.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn from_iter() {
        let vec = vec![false, true, false, true, false];
        let boolean_array = vec.iter().collect::<BooleanArray<false>>();
        assert_eq!(vec, boolean_array.into_iter().collect::<Vec<bool>>());

        let vec = vec![Some(false), Some(true), None, Some(true), None];
        let boolean_array = vec.iter().collect::<BooleanArray<true>>();
        assert_eq!(
            vec,
            boolean_array.into_iter().collect::<Vec<Option<bool>>>()
        );
    }
}
