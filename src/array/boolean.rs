use crate::{Array, ArrayType, Bitmap, Nullable, Validity, DEFAULT_ALIGNMENT};
use std::ops::Deref;

/// Array with boolean values.
///
/// Values are stored using single bits in a [Bitmap].
#[derive(Debug, PartialEq, Eq)]
pub struct BooleanArray<const N: bool, const A: usize = DEFAULT_ALIGNMENT>(Validity<Bitmap<A>, N>);

impl<const N: bool, const A: usize> Array for BooleanArray<N, A> {
    type Item<'a> = bool;
}

impl ArrayType for bool {
    type Item<'a> = bool;
    type Array<T, const N: bool, const A: usize> = BooleanArray<false, A>;
}

impl Deref for BooleanArray<false> {
    type Target = Validity<Bitmap, false>;

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

impl<T, const N: bool, const A: usize> FromIterator<T> for BooleanArray<N, A>
where
    Validity<Bitmap<A>, N>: FromIterator<T>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Length;

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
