//! A collection that flattens an inner collection.

use core::{
    array, fmt,
    iter::{self, Map, RepeatN, Zip},
    ops::{Deref, Range},
};

use crate::{
    collection::{
        AllocError, ChildRef, Collection, CollectionAlloc, CollectionAllocIn, CollectionRealloc,
        owned::IntoOwned,
    },
    length::Length,
};

/// A collection that flattens an inner collection and exposes its items as
/// arrays with N items.
///
/// Arrow fixed-size lists are one flat child collection plus a constant width,
/// rather than a collection of separately allocated arrays. `Flatten` presents
/// that physical form as logical `[T; N]` items.
///
/// # Examples
///
/// ```
/// use narrow::collection::{Collection, flatten::Flatten};
///
/// let values = [[1, 2], [3, 4]].into_iter().collect::<Flatten<Vec<_>, 2>>();
/// assert_eq!(values.owned(1), Some([3, 4]));
/// ```
///
/// The chunk size `N` must be non-zero. Constructing a `Flatten` with `N = 0`
/// is a compile-time error:
///
/// ```compile_fail
/// use narrow::collection::flatten::Flatten;
///
/// let values = Flatten::<Vec<i32>, 0>::default();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Flatten<C: Collection, const N: usize>(C);

impl<C: Collection, const N: usize> Flatten<C, N> {
    /// Wraps `child`, rejecting `N = 0` at compile time.
    ///
    /// Every constructor goes through this function, so no `Flatten` with a
    /// zero chunk size exists and dividing by `N` can not panic.
    const fn new(child: C) -> Self {
        const { assert!(N > 0, "the chunk size N of Flatten must be non-zero") };
        Self(child)
    }
}

impl<C: Collection + Default, const N: usize> Default for Flatten<C, N> {
    fn default() -> Self {
        Self::new(C::default())
    }
}

/// Error returned by [`Flatten::try_from_parts`].
///
/// Validation happens when raw parts enter the safe representation. This lets
/// later indexing assume that every logical item has a complete group of
/// child values. A zero chunk size `N` is rejected at compile time.
///
/// # Examples
///
/// ```
/// use narrow::collection::flatten::{Flatten, FlattenError};
///
/// let error = Flatten::<Vec<_>, 2>::try_from_parts(vec![1]).unwrap_err();
/// assert_eq!(error, FlattenError::NotMultiple { len: 1, n: 2 });
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlattenError {
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
    /// Returns a [`FlattenError`] when the length of the child collection is
    /// not a multiple of `N`.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::{Collection, flatten::Flatten};
    ///
    /// let values = Flatten::<Vec<_>, 2>::try_from_parts(vec![1, 2]).unwrap();
    /// assert_eq!(values.owned(0), Some([1, 2]));
    /// ```
    pub fn try_from_parts(child: C) -> Result<Self, FlattenError> {
        let len = child.len();
        if len.is_multiple_of(N) {
            Ok(Self::new(child))
        } else {
            Err(FlattenError::NotMultiple { len, n: N })
        }
    }

    /// Constructs a [`Flatten`] from a `child` collection, panicking instead
    /// of returning a [`FlattenError`].
    ///
    /// This is [`Flatten::try_from_parts`] without the `Result`, for callers
    /// that know their parts are valid.
    ///
    /// # Panics
    ///
    /// Panics when the length of the child collection is not a multiple of
    /// `N`.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::{Collection, flatten::Flatten};
    ///
    /// let values = Flatten::<Vec<_>, 2>::from_parts(vec![1, 2]);
    /// assert_eq!(values.owned(0), Some([1, 2]));
    /// ```
    #[must_use]
    pub fn from_parts(child: C) -> Self {
        assert!(
            child.len().is_multiple_of(N),
            "child length is not a multiple of N"
        );
        Self::new(child)
    }

    /// Returns the child collection of this [`Flatten`].
    ///
    /// This is the inverse of [`Flatten::try_from_parts`].
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::flatten::Flatten;
    ///
    /// let values = Flatten::<Vec<_>, 2>::try_from_parts(vec![1, 2]).unwrap();
    /// assert_eq!(values.into_parts(), [1, 2]);
    /// ```
    #[must_use]
    pub fn into_parts(self) -> C {
        self.0
    }
}

impl<C: Collection, const N: usize> ChildRef for Flatten<C, N> {
    type Child = C;

    fn child_ref(&self) -> &Self::Child {
        &self.0
    }
}

impl<C: Collection, const N: usize> Length for Flatten<C, N> {
    fn len(&self) -> usize {
        self.0.len().strict_div(N)
    }
}

impl<C: CollectionRealloc, const N: usize> Extend<[C::Owned; N]> for Flatten<C, N> {
    fn extend<I: IntoIterator<Item = [C::Owned; N]>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().flatten());
    }
}

impl<C: CollectionAlloc, const N: usize> FromIterator<[C::Owned; N]> for Flatten<C, N> {
    fn from_iter<T: IntoIterator<Item = [C::Owned; N]>>(iter: T) -> Self {
        Self::new(iter.into_iter().flatten().collect())
    }
}

impl<C: Collection, const N: usize> Collection for Flatten<C, N> {
    type View<'collection>
        = FlattenView<'collection, C, N>
    where
        Self: 'collection;

    type Owned = [C::Owned; N];

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        // `len` is the child's length divided by `N`, so every child index of
        // an in-bounds chunk is in bounds.
        (index < self.len()).then(|| {
            FlattenView(array::from_fn(|idx| {
                self.0
                    .view(index.strict_mul(N).strict_add(idx))
                    .expect("chunk is within the child")
            }))
        })
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

impl<C: CollectionAllocIn, const N: usize> CollectionAllocIn for Flatten<C, N> {
    type Alloc = C::Alloc;

    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self {
        Self::new(C::with_capacity_in(capacity.strict_mul(N), alloc))
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, alloc: Self::Alloc) -> Self {
        Self::new(C::from_iter_in(iter.into_iter().flatten(), alloc))
    }

    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, AllocError> {
        let child_capacity = capacity.checked_mul(N).ok_or(AllocError)?;
        C::try_with_capacity_in(child_capacity, alloc).map(Self::new)
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, AllocError> {
        C::try_from_iter_in(iter.into_iter().flatten(), alloc).map(Self::new)
    }
}

impl<C: CollectionRealloc, const N: usize> CollectionRealloc for Flatten<C, N> {
    fn allocator(&self) -> Self::Alloc {
        self.0.allocator()
    }

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
///
/// The array of child views is assembled by value, so reading a fixed-size list
/// does not materialize or allocate its owned child items.
///
/// # Examples
///
/// ```
/// use narrow::collection::{Collection, flatten::{Flatten, FlattenView}};
///
/// let values = [[1, 2]].into_iter().collect::<Flatten<Vec<_>, 2>>();
/// let view: FlattenView<'_, Vec<i32>, 2> = values.view(0).unwrap();
/// assert_eq!(*view, [1, 2]);
/// ```
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
///
/// This is the consuming counterpart to [`FlattenView`]: it reconstructs each
/// owned array directly from the flat child iterator.
///
/// # Examples
///
/// ```
/// use narrow::collection::{Collection, flatten::{ArrayChunks, Flatten}};
///
/// let values = [[1, 2]].into_iter().collect::<Flatten<Vec<_>, 2>>();
/// let mut chunks: ArrayChunks<2, _> = values.into_iter_owned();
/// assert_eq!(chunks.next(), Some([1, 2]));
/// ```
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
    fn view_out_of_bounds() {
        let flatten = [[1, 2], [3, 4]].into_iter().collect::<Flatten<Vec<_>, 2>>();
        assert!(flatten.view(1).is_some());
        assert!(flatten.view(2).is_none());
        assert!(flatten.view(usize::MAX).is_none());
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
    fn from_parts() {
        let values = Flatten::<Vec<i32>, 2>::from_parts(alloc::vec![1, 2, 3, 4]);
        assert_eq!(values.owned(1), Some([3, 4]));
    }

    #[test]
    #[should_panic(expected = "child length is not a multiple of N")]
    fn from_parts_not_multiple() {
        let _ = Flatten::<Vec<i32>, 2>::from_parts(alloc::vec![1, 2, 3]);
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
