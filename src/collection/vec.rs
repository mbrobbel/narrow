extern crate alloc;

use crate::collection::slice::SliceViews;
use alloc::vec::Vec;
use core::alloc::{AllocError, Allocator, Layout};

use crate::collection::{
    Collection, CollectionAllocIn, CollectionRealloc, TryReserveError, view::AsView,
};

impl<T: for<'any> AsView<'any>, A: Allocator> Collection for Vec<T, A> {
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

    type IntoIter = <Self as IntoIterator>::IntoIter;

    fn into_iter_owned(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<T: for<'any> AsView<'any>, A: Allocator + Clone> CollectionAllocIn for Vec<T, A> {
    type Alloc = A;

    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self {
        Vec::with_capacity_in(capacity, alloc)
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, alloc: Self::Alloc) -> Self {
        let items = iter.into_iter();
        let (capacity, _) = items.size_hint();
        let mut collection = Vec::with_capacity_in(capacity, alloc);
        collection.extend(items);
        collection
    }

    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, TryReserveError> {
        let mut collection = Vec::new_in(alloc);
        try_reserve_exact(&mut collection, capacity)?;
        Ok(collection)
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, TryReserveError> {
        let mut items = iter.into_iter();
        let (capacity, _) = items.size_hint();
        let mut collection = <Self as CollectionAllocIn>::try_with_capacity_in(capacity, alloc)?;
        for item in items.by_ref() {
            try_reserve(&mut collection, 1)?;
            collection.push(item);
        }
        Ok(collection)
    }
}

/// Reserves room for at least `additional` more items in `values`.
///
/// # Errors
///
/// Returns [`TryReserveError::CapacityOverflow`] when the capacity would
/// exceed `usize::MAX` items or `isize::MAX` bytes, and
/// [`TryReserveError::AllocError`] when the allocator fails.
pub(crate) fn try_reserve<T, A: Allocator>(
    values: &mut Vec<T, A>,
    additional: usize,
) -> Result<(), TryReserveError> {
    // Amortized growth can ask for more than is representable even when
    // `additional` fits, so retry with an exact reservation before reporting.
    values
        .try_reserve(additional)
        .or_else(|_| try_reserve_exact(values, additional))
}

/// Reserves room for exactly `additional` more items in `values`.
///
/// # Errors
///
/// As for [`try_reserve`].
pub(crate) fn try_reserve_exact<T, A: Allocator>(
    values: &mut Vec<T, A>,
    additional: usize,
) -> Result<(), TryReserveError> {
    values
        .try_reserve_exact(additional)
        .map_err(|_| reserve_error::<T>(values.len(), additional))
}

/// Classifies a failure to reserve exactly `additional` items beyond `len`
/// in a `Vec<T, A>`, as std does not expose the kind of its error on stable.
fn reserve_error<T>(len: usize, additional: usize) -> TryReserveError {
    let representable = len
        .checked_add(additional)
        .is_some_and(|capacity| Layout::array::<T>(capacity).is_ok());
    if representable {
        TryReserveError::AllocError(AllocError)
    } else {
        TryReserveError::CapacityOverflow
    }
}

impl<T: for<'any> AsView<'any>, A: Allocator + Clone> CollectionRealloc for Vec<T, A> {
    fn allocator(&self) -> Self::Alloc {
        Vec::allocator(self).clone()
    }

    fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        try_reserve(self, additional)
    }

    fn try_extend<I: IntoIterator<Item = Self::Owned>>(
        &mut self,
        iter: I,
    ) -> Result<(), TryReserveError> {
        let len = self.len();
        let items = iter.into_iter();
        let (additional, _) = items.size_hint();
        try_reserve(self, additional)?;

        for item in items {
            // `Vec::push_within_capacity` would express this directly, but is unstable:
            // https://doc.rust-lang.org/stable/std/vec/struct.Vec.html#method.push_within_capacity
            if self.len() == self.capacity()
                && let Err(error) = try_reserve(self, 1)
            {
                Vec::truncate(self, len);
                return Err(error);
            }
            self.push(item);
        }
        Ok(())
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
    use alloc::alloc::Global;

    use super::*;
    use crate::collection::CollectionAlloc;

    #[test]
    fn alloc_in() {
        let reserved = <Vec<u32> as CollectionAlloc>::with_capacity(4);
        assert!(reserved.capacity() >= 4);

        let collection = <Vec<u32> as CollectionAllocIn>::try_from_iter_in([1, 2, 3, 4], Global)
            .expect("allocation succeeds");
        assert_eq!(collection, [1, 2, 3, 4]);

        let infallible = <Vec<u32> as CollectionAllocIn>::from_iter_in([5, 6, 7, 8], Global);
        assert_eq!(infallible, [5, 6, 7, 8]);
    }

    #[test]
    fn alloc_in_capacity_overflow() {
        let mut collection = alloc::vec![1_u32, 2, 3, 4];
        assert_eq!(
            <Vec<u32> as CollectionAllocIn>::try_with_capacity_in(usize::MAX, Global),
            Err(TryReserveError::CapacityOverflow)
        );
        assert_eq!(
            CollectionRealloc::try_reserve(&mut collection, usize::MAX),
            Err(TryReserveError::CapacityOverflow)
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

    #[test]
    fn try_extend_capacity_overflow() {
        let mut collection = alloc::vec![1_u32, 2, 3, 4];
        let items = core::iter::repeat_n(5, usize::MAX);
        assert_eq!(
            CollectionRealloc::try_extend(&mut collection, items),
            Err(TryReserveError::CapacityOverflow)
        );
        assert_eq!(collection, [1, 2, 3, 4]);
    }

    #[test]
    #[should_panic(expected = "capacity overflow")]
    fn alloc_in_capacity_overflow_panics() {
        let _ = <Vec<u32> as CollectionAllocIn>::with_capacity_in(usize::MAX, Global);
    }

    #[test]
    #[should_panic(expected = "capacity overflow")]
    fn reserve_capacity_overflow_panics() {
        let mut collection = alloc::vec![1_u32, 2, 3, 4];
        CollectionRealloc::reserve(&mut collection, usize::MAX);
    }
}
