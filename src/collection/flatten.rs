//! A collection that flattens an inner collection.

use std::{
    array,
    iter::{self, Map, RepeatN, Zip},
    ops::{Deref, Range},
};

use crate::{
    collection::{Collection, CollectionAlloc, CollectionRealloc, owned::IntoOwned},
    length::Length,
};

/// A collection that flattens an inner collection and exposes its items as
/// arrays with N items.
#[derive(Clone, Copy, Debug, Default)]
pub struct Flatten<C: Collection, const N: usize>(C);

impl<C: Collection, const N: usize> Length for Flatten<C, N> {
    fn len(&self) -> usize {
        self.0.len() / N
    }
}

impl<C: CollectionRealloc, const N: usize> Extend<[C::Owned; N]> for Flatten<C, N> {
    fn extend<I: IntoIterator<Item = [C::Owned; N]>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();
        self.reserve(upper_bound.unwrap_or(lower_bound));
        self.0.extend(iter.flatten());
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
            self.0.view(index * N + idx).expect("out of bounds")
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
        Self(C::with_capacity(capacity * N))
    }
}

impl<C: CollectionRealloc, const N: usize> CollectionRealloc for Flatten<C, N> {
    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional.checked_mul(N).expect("overflow"));
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
        (self.0.len() != 0).then(|| {
            let mut items = self.0.by_ref().take(N);
            array::from_fn(|_| items.next().expect("out of bounds"))
        })
    }
}

impl<const N: usize, I: ExactSizeIterator> ExactSizeIterator for ArrayChunks<N, I> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::collection::tests::round_trip;

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
    fn collection() {
        round_trip::<Flatten<Vec<u8>, _>, _>([[1, 2], [3, 4]]);
        round_trip::<Flatten<Vec<u8>, _>, _>([[1, 2, 3, 4], [5, 6, 7, 8]]);
        round_trip::<Flatten<Flatten<Vec<u8>, _>, _>, _>([[[1, 2], [3, 4]], [[5, 6], [7, 8]]]);
    }
}
