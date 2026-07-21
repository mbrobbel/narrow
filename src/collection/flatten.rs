//! A collection that flattens an inner collection.

use core::{
    array, fmt,
    iter::{self, Map, RepeatN, Zip},
    ops::{Deref, Range},
};

use crate::{
    collection::{
        AllocError, Collection, CollectionAlloc, CollectionAllocIn, CollectionRealloc,
        owned::IntoOwned,
    },
    length::Length,
};

/// A collection that flattens an inner collection and exposes its items as
/// arrays with N items.
#[derive(Clone, Copy, Debug, Default)]
pub struct Flatten<C: Collection, const N: usize>(C);

/// Error returned by [`Flatten::try_from_parts`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlattenError {
    /// The chunk size `N` is zero.
    ZeroChunkSize,
    /// The child collection length is not a multiple of `N`.
    NotMultiple {
        /// The length of the child collection.
        len: usize,
        /// The chunk size `N`.
        n: usize,
    },
}

impl fmt::Display for FlattenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ZeroChunkSize => write!(f, "chunk size N must be non-zero"),
            Self::NotMultiple { len, n } => {
                write!(f, "child length ({len}) is not a multiple of N ({n})")
            }
        }
    }
}

impl core::error::Error for FlattenError {}

impl<C: Collection, const N: usize> Flatten<C, N> {
    /// Constructs a [`Flatten`] from a `child` collection.
    ///
    /// # Errors
    ///
    /// Returns a [`FlattenError`] when `N` is zero or when the length of the
    /// child collection is not a multiple of `N`.
    pub fn try_from_parts(child: C) -> Result<Self, FlattenError> {
        match child.len().checked_rem(N) {
            None => Err(FlattenError::ZeroChunkSize),
            Some(0) => Ok(Self(child)),
            Some(_) => Err(FlattenError::NotMultiple {
                len: child.len(),
                n: N,
            }),
        }
    }

    /// Returns the child collection of this [`Flatten`].
    ///
    /// This is the inverse of [`Flatten::try_from_parts`].
    #[must_use]
    pub fn into_parts(self) -> C {
        self.0
    }
}

impl<C: Collection, const N: usize> Length for Flatten<C, N> {
    fn len(&self) -> usize {
        self.0.len().strict_div(N)
    }
}

impl<C: CollectionRealloc, const N: usize> Extend<[C::Owned; N]> for Flatten<C, N> {
    fn extend<I: IntoIterator<Item = [C::Owned; N]>>(&mut self, iter: I) {
        let into_iter = iter.into_iter();
        let (lower_bound, upper_bound) = into_iter.size_hint();
        self.reserve(upper_bound.unwrap_or(lower_bound));
        self.0.extend(into_iter.flatten());
    }
}

impl<C: CollectionAlloc, const N: usize> FromIterator<[C::Owned; N]> for Flatten<C, N> {
    fn from_iter<T: IntoIterator<Item = [C::Owned; N]>>(iter: T) -> Self {
        Self(iter.into_iter().flatten().collect())
    }
}

impl<C: Collection, const N: usize> Collection for Flatten<C, N> {
    type View<'collection>
        = FlattenView<'collection, C, N>
    where
        Self: 'collection;

    type Owned = [C::Owned; N];

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        Some(FlattenView(array::from_fn(|idx| {
            self.0
                .view(index.strict_mul(N).strict_add(idx))
                .expect("out of bounds")
        })))
    }

    type Iter<'collection>
        = Map<
        Zip<Range<usize>, RepeatN<&'collection Self>>,
        fn((usize, &'collection Self)) -> FlattenView<'collection, C, N>,
    >
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        (0..self.len())
            .zip(iter::repeat_n(self, self.len()))
            .map(|(idx, collection)| collection.view(idx).expect("out of bounds"))
    }

    type IntoIter = ArrayChunks<N, C::IntoIter>;

    fn into_iter_owned(self) -> Self::IntoIter {
        ArrayChunks(self.0.into_iter_owned())
    }
}

impl<C: CollectionAlloc, const N: usize> CollectionAlloc for Flatten<C, N> {
    fn with_capacity(capacity: usize) -> Self {
        Self(C::with_capacity(capacity.strict_mul(N)))
    }
}

impl<C: CollectionAllocIn, const N: usize> CollectionAllocIn for Flatten<C, N> {
    type Alloc = C::Alloc;

    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self {
        Self(C::with_capacity_in(capacity.strict_mul(N), alloc))
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, alloc: Self::Alloc) -> Self {
        Self(C::from_iter_in(iter.into_iter().flatten(), alloc))
    }

    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, AllocError> {
        let child_capacity = capacity.checked_mul(N).ok_or(AllocError)?;
        C::try_with_capacity_in(child_capacity, alloc).map(Self)
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, AllocError> {
        C::try_from_iter_in(iter.into_iter().flatten(), alloc).map(Self)
    }
}

impl<C: CollectionRealloc, const N: usize> CollectionRealloc for Flatten<C, N> {
    fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError> {
        let child_additional = additional.checked_mul(N).ok_or(AllocError)?;
        self.0.try_reserve(child_additional)
    }

    fn try_extend<I: IntoIterator<Item = Self::Owned>>(
        &mut self,
        iter: I,
    ) -> Result<(), AllocError> {
        self.0.try_extend(iter.into_iter().flatten())
    }

    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional.strict_mul(N));
    }

    fn truncate(&mut self, len: usize) {
        if len < self.len() {
            self.0.truncate(len.strict_mul(N));
        }
    }
}

/// A view of `Flatten`. This is an array with N views of the inner collection.
#[derive(Debug)]
pub struct FlattenView<'collection, C: Collection + 'collection, const N: usize>(
    [C::View<'collection>; N],
);

impl<'collection, C: Collection + 'collection, const N: usize> Clone
    for FlattenView<'collection, C, N>
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'collection, C: Collection + 'collection, const N: usize> Copy
    for FlattenView<'collection, C, N>
{
}

impl<'collection, C: Collection + 'collection, const N: usize> Deref
    for FlattenView<'collection, C, N>
{
    type Target = [C::View<'collection>; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: Collection, const N: usize> IntoOwned<[C::Owned; N]> for FlattenView<'_, C, N> {
    fn into_owned(self) -> [C::Owned; N] {
        self.0.map(IntoOwned::into_owned)
    }
}

/// An iterator over N elements of the inner iterator at a time.
#[derive(Debug, Clone, Copy)]
pub struct ArrayChunks<const N: usize, I: ExactSizeIterator>(I);

impl<const N: usize, I: ExactSizeIterator> Iterator for ArrayChunks<N, I> {
    type Item = [I::Item; N];

    fn next(&mut self) -> Option<Self::Item> {
        let first = self.0.next()?;
        let mut items = array::from_fn(|_| None);
        items[0] = Some(first);
        items.iter_mut().take(N).skip(1).for_each(|item| {
            *item = self.0.next();
        });
        Some(items.map(|item| item.expect("out of bounds")))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<const N: usize, I: ExactSizeIterator> ExactSizeIterator for ArrayChunks<N, I> {
    fn len(&self) -> usize {
        self.0.len().strict_div(N)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use crate::collection::tests::round_trip;
    use alloc::vec::Vec;

    use super::*;

    #[test]
    fn flattens() {
        let flatten = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]]
            .into_iter()
            .collect::<Flatten<Flatten<Vec<_>, _>, _>>();
        assert_eq!(flatten.0.0, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(
            flatten
                .view(0)
                .as_deref()
                .map(|items| items.map(|item| *item)),
            Some([[1, 2], [3, 4]])
        );
        assert_eq!(
            flatten
                .view(1)
                .as_deref()
                .map(|items| items.map(|item| *item)),
            Some([[5, 6], [7, 8]])
        );
    }

    #[test]
    fn try_from_parts() {
        let flatten =
            Flatten::<Vec<i32>, 2>::try_from_parts(alloc::vec![1, 2, 3, 4]).expect("valid parts");
        assert_eq!(flatten.len(), 2);
        assert_eq!(flatten.into_parts(), alloc::vec![1, 2, 3, 4]);
    }

    #[test]
    fn try_from_parts_not_multiple() {
        let error = Flatten::<Vec<i32>, 2>::try_from_parts(alloc::vec![1, 2, 3])
            .expect_err("not a multiple");
        assert_eq!(error, FlattenError::NotMultiple { len: 3, n: 2 });
    }

    #[test]
    fn try_from_parts_zero_chunk() {
        let error = Flatten::<Vec<i32>, 0>::try_from_parts(alloc::vec![1, 2, 3])
            .expect_err("zero chunk size");
        assert_eq!(error, FlattenError::ZeroChunkSize);
    }

    #[test]
    fn truncate() {
        let mut flatten = [[1, 2], [3, 4], [5, 6]]
            .into_iter()
            .collect::<Flatten<Vec<i32>, 2>>();
        assert_eq!(flatten.len(), 3);
        flatten.truncate(1);
        assert_eq!(flatten.len(), 1);
        assert_eq!(flatten.owned(0), Some([1, 2]));
    }

    #[test]
    fn truncate_beyond_len_is_noop() {
        let mut flatten = [[1, 2], [3, 4]]
            .into_iter()
            .collect::<Flatten<Vec<i32>, 2>>();
        // Must not overflow `len * N`.
        flatten.truncate(usize::MAX);
        assert_eq!(flatten.len(), 2);
        assert_eq!(flatten.owned(1), Some([3, 4]));
    }

    #[test]
    fn collection() {
        round_trip::<Flatten<Vec<_>, _>, _>([[1, 2], [3, 4]]);
        round_trip::<Flatten<Vec<_>, _>, _>([[1, 2, 3, 4], [5, 6, 7, 8]]);
        round_trip::<Flatten<Flatten<Vec<_>, _>, _>, _>([[[1, 2], [3, 4]], [[5, 6], [7, 8]]]);
    }
}
