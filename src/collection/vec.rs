extern crate alloc;

use crate::collection::slice::SliceViews;
use alloc::vec::{self, Vec};

use crate::collection::{
    AllocError, Collection, CollectionAllocIn, CollectionRealloc, view::AsView,
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
        = SliceViews<'collection, T>
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        SliceViews::new(self)
    }

    type IntoIter = vec::IntoIter<T>;

    fn into_iter_owned(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
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
        let mut items = iter.into_iter().fuse();
        let (capacity, _) = items.size_hint();
        let mut collection = <Self as CollectionAllocIn>::try_with_capacity_in(capacity, ())?;
        // At most `capacity` items fit in the reserved capacity, so this never
        // reallocates.
        collection.extend(items.by_ref().take(capacity));
        try_push_remaining(&mut collection, items)?;
        Ok(collection)
    }
}

/// Pushes `items` one at a time, reserving fallibly when `collection` is full.
///
/// # Errors
///
/// Returns [`AllocError`] when reserving fails; the items pushed before the
/// failure are kept.
fn try_push_remaining<T>(
    collection: &mut Vec<T>,
    items: impl Iterator<Item = T>,
) -> Result<(), AllocError> {
    for item in items {
        // `Vec::push_within_capacity` would express this directly, but is nightly-only:
        // https://doc.rust-lang.org/stable/std/vec/struct.Vec.html#method.push_within_capacity
        if collection.len() == collection.capacity() {
            collection.try_reserve(1).map_err(|_| AllocError)?;
        }
        collection.push(item);
    }
    Ok(())
}

impl<T: for<'any> AsView<'any>> CollectionRealloc for Vec<T> {
    fn allocator(&self) -> Self::Alloc {}

    fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError> {
        Vec::try_reserve(self, additional).map_err(|_| AllocError)
    }

    fn try_extend<I: IntoIterator<Item = Self::Owned>>(
        &mut self,
        iter: I,
    ) -> Result<(), AllocError> {
        let len = self.len();
        let mut items = iter.into_iter().fuse();
        let (additional, _) = items.size_hint();
        Vec::try_reserve(self, additional).map_err(|_| AllocError)?;
        // At most `additional` items fit in the reserved capacity, so this
        // never reallocates.
        self.extend(items.by_ref().take(additional));

        try_push_remaining(self, items).inspect_err(|_| Vec::truncate(self, len))
    }

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
    use crate::collection::CollectionAlloc;

    #[test]
    fn alloc_in() {
        let reserved = <Vec<u32> as CollectionAlloc>::with_capacity(4);
        assert!(reserved.capacity() >= 4);

        let collection = <Vec<u32> as CollectionAllocIn>::try_from_iter_in([1, 2, 3, 4], ())
            .expect("allocation succeeds");
        assert_eq!(collection, [1, 2, 3, 4]);

        let infallible = <Vec<u32> as CollectionAllocIn>::from_iter_in([5, 6, 7, 8], ());
        assert_eq!(infallible, [5, 6, 7, 8]);
    }

    #[test]
    fn alloc_in_capacity_overflow() {
        let mut collection = alloc::vec![1_u32, 2, 3, 4];
        assert_eq!(
            <Vec<u32> as CollectionAllocIn>::try_with_capacity_in(usize::MAX, ()),
            Err(AllocError)
        );
        assert_eq!(
            CollectionRealloc::try_reserve(&mut collection, usize::MAX),
            Err(AllocError)
        );
        assert_eq!(collection, [1, 2, 3, 4]);
    }

    #[test]
    fn try_extend() {
        let mut collection = alloc::vec![1_u32, 2];
        CollectionRealloc::try_extend(&mut collection, (3..=5).filter(|_| true))
            .expect("allocation succeeds");
        assert_eq!(collection, [1, 2, 3, 4, 5]);
    }

    /// Yields `items` while reporting `lower` as its size hint's lower bound,
    /// which may overstate the number of remaining items.
    struct Hinted<I> {
        items: I,
        lower: usize,
    }

    impl<I: Iterator<Item = u32>> Iterator for Hinted<I> {
        type Item = u32;

        fn next(&mut self) -> Option<u32> {
            self.items.next()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.lower, None)
        }
    }

    #[test]
    fn try_extend_size_hints() {
        for (len, lower) in [(0, 0), (0, 3), (5, 0), (5, 2), (5, 5), (5, 8)] {
            let expected = (0..len).collect::<Vec<_>>();

            let mut collection = alloc::vec![7_u32];
            CollectionRealloc::try_extend(
                &mut collection,
                Hinted {
                    items: 0..len,
                    lower,
                },
            )
            .expect("allocation succeeds");
            assert_eq!(collection[..1], [7]);
            assert_eq!(collection[1..], expected);

            let collected = <Vec<u32> as CollectionAllocIn>::try_from_iter_in(
                Hinted {
                    items: 0..len,
                    lower,
                },
                (),
            )
            .expect("allocation succeeds");
            assert_eq!(collected, expected);
        }
    }

    #[test]
    fn try_extend_stops_at_first_none() {
        // Yields `1`, then `None`, then `2` if polled again.
        let items = || {
            let mut items = [Some(1_u32), None, Some(2)].into_iter();
            Hinted {
                items: core::iter::from_fn(move || items.next().flatten()),
                lower: 3,
            }
        };

        let mut collection = alloc::vec![7_u32];
        CollectionRealloc::try_extend(&mut collection, items()).expect("allocation succeeds");
        assert_eq!(collection, [7, 1]);

        let collected = <Vec<u32> as CollectionAllocIn>::try_from_iter_in(items(), ())
            .expect("allocation succeeds");
        assert_eq!(collected, [1]);
    }

    #[test]
    fn try_extend_capacity_overflow() {
        let mut collection = alloc::vec![1_u32, 2, 3, 4];
        let items = core::iter::repeat_n(5, usize::MAX);
        assert_eq!(
            CollectionRealloc::try_extend(&mut collection, items),
            Err(AllocError)
        );
        assert_eq!(collection, [1, 2, 3, 4]);
    }

    #[test]
    #[should_panic(expected = "capacity overflow")]
    fn alloc_in_capacity_overflow_panics() {
        let _ = <Vec<u32> as CollectionAllocIn>::with_capacity_in(usize::MAX, ());
    }

    #[test]
    #[should_panic(expected = "capacity overflow")]
    fn reserve_capacity_overflow_panics() {
        let mut collection = alloc::vec![1_u32, 2, 3, 4];
        CollectionRealloc::reserve(&mut collection, usize::MAX);
    }
}
