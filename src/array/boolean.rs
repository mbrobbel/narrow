use crate::{Array, ArrayIndex, ArrayType, Bitmap, Nullable, Validity};
use std::ops::Deref;

/// Array with boolean values.
///
/// Values are stored using single bits in a [Bitmap].
#[derive(Debug, PartialEq, Eq)]
pub struct BooleanArray<const N: bool>(Validity<Bitmap, N>);

impl<const N: bool> Array for BooleanArray<N> {
    type Validity = Validity<Bitmap, N>;

    fn validity(&self) -> &Self::Validity {
        &self.0
    }
}

impl ArrayType for bool {
    type Array = BooleanArray<false>;
}

impl ArrayType for Option<bool> {
    type Array = BooleanArray<true>;
}

impl ArrayIndex<usize> for BooleanArray<false> {
    type Output = bool;

    fn index(&self, index: usize) -> Self::Output {
        self.0[index]
    }
}

impl ArrayIndex<usize> for BooleanArray<true> {
    type Output = Option<bool>;

    fn index(&self, index: usize) -> Self::Output {
        self.0.index(index)
    }
}

impl Deref for BooleanArray<false> {
    type Target = Bitmap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BooleanArray<true> {
    type Target = Nullable<Bitmap>;

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

impl FromIterator<Option<bool>> for BooleanArray<true> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<bool>>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<'a> IntoIterator for &'a BooleanArray<false> {
    type Item = bool;
    type IntoIter = <&'a Bitmap as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a BooleanArray<true> {
    type Item = Option<bool>;
    type IntoIter = <&'a Nullable<Bitmap> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deref() {
        let vec = vec![true, false, true, false, true, true];
        let boolean_array: BooleanArray<false> = vec.iter().copied().collect();
        assert_eq!(boolean_array.len(), vec.len());
        assert!(&boolean_array[0]);
        assert!(!&boolean_array[1]);
        assert_eq!(vec, boolean_array.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn from_iter() {
        let vec = vec![false, true, false, true, false];
        let boolean_array = vec.iter().copied().collect::<BooleanArray<false>>();
        assert_eq!(vec, boolean_array.into_iter().collect::<Vec<bool>>());

        let vec = vec![Some(false), Some(true), None, Some(true), None];
        let boolean_array = vec.iter().copied().collect::<BooleanArray<true>>();
        assert_eq!(
            vec,
            boolean_array.into_iter().collect::<Vec<Option<bool>>>()
        );
    }
}
