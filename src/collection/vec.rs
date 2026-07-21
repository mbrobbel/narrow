extern crate alloc;

use alloc::vec::{self, Vec};
use core::{iter::Map, slice};

use crate::collection::{
    AllocError, Collection, CollectionAlloc, CollectionAllocIn, CollectionRealloc, view::AsView,
};

impl<T: for<'any> AsView<'any>> Collection for Vec<T> {
    type View<'collection>
        = <T as AsView<'collection>>::View
    where
        Self: 'collection;

    type Owned = T;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.get(index).map(AsView::as_view)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> <T as AsView<'collection>>::View>
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self).map(AsView::as_view)
    }

    type IntoIter = vec::IntoIter<T>;

    fn into_iter_owned(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<T: for<'any> AsView<'any>> CollectionAlloc for Vec<T> {
    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity)
    }
}

// The analogous fallible and allocator-aware `Vec` constructors are nightly-only:
// https://doc.rust-lang.org/stable/std/vec/struct.Vec.html#method.try_with_capacity
// https://doc.rust-lang.org/stable/std/vec/struct.Vec.html#method.with_capacity_in
// This stable implementation represents the global allocator as `()`.
impl<T: for<'any> AsView<'any>> CollectionAllocIn for Vec<T> {
    type Alloc = ();

    fn with_capacity_in(capacity: usize, (): Self::Alloc) -> Self {
        Vec::with_capacity(capacity)
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, (): Self::Alloc) -> Self {
        iter.into_iter().collect()
    }

    fn try_with_capacity_in(capacity: usize, (): Self::Alloc) -> Result<Self, AllocError> {
        let mut collection = Vec::new();
        collection
            .try_reserve_exact(capacity)
            .map_err(|_| AllocError)?;
        Ok(collection)
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        (): Self::Alloc,
    ) -> Result<Self, AllocError> {
        let mut items = iter.into_iter();
        let (capacity, _) = items.size_hint();
        let mut collection = <Self as CollectionAllocIn>::try_with_capacity_in(capacity, ())?;
        for item in items.by_ref() {
            collection.try_reserve(1).map_err(|_| AllocError)?;
            collection.push(item);
        }
        Ok(collection)
    }
}

impl<T: for<'any> AsView<'any>> CollectionRealloc for Vec<T> {
    fn reserve(&mut self, additional: usize) {
        Vec::reserve(self, additional);
    }

    fn truncate(&mut self, len: usize) {
        Vec::truncate(self, len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_in() {
        let collection = <Vec<u32> as CollectionAllocIn>::try_from_iter_in([1, 2, 3, 4], ())
            .expect("allocation succeeds");
        assert_eq!(collection, [1, 2, 3, 4]);

        let infallible = <Vec<u32> as CollectionAllocIn>::from_iter_in([5, 6, 7, 8], ());
        assert_eq!(infallible, [5, 6, 7, 8]);
    }

    #[test]
    fn alloc_in_capacity_overflow() {
        assert_eq!(
            <Vec<u32> as CollectionAllocIn>::try_with_capacity_in(usize::MAX, ()),
            Err(AllocError)
        );
    }
}
