//! Collections of items.

pub mod owned;
pub mod view;

pub mod arc;
pub mod array;
pub mod r#box;
pub mod rc;
pub mod slice;
pub mod vec;

pub mod flatten;

use core::fmt;

use crate::{collection::owned::IntoOwned, length::Length};

/// A collection of items.
pub trait Collection: Length {
    /// Borrowed view of an item in this collection
    type View<'collection>: Copy + IntoOwned<Self::Owned> + 'collection
    where
        Self: 'collection;

    /// Owned items in this collection
    type Owned;

    /// Returns a reference to an item at the given index in this collection or
    /// `None` if out of bounds.
    fn view(&self, index: usize) -> Option<Self::View<'_>>;

    /// Returns an owned item at the given index in this collection or `None`
    /// if out of bounds.
    fn owned(&self, index: usize) -> Option<Self::Owned> {
        self.view(index).map(IntoOwned::into_owned)
    }

    /// Iterator over referenced items in this collection.
    type Iter<'collection>: Iterator<Item = Self::View<'collection>>
    where
        Self: 'collection;

    /// Returns an iterator over references to the items in this collection.
    fn iter_views(&self) -> Self::Iter<'_>;

    /// Iterator over owned items in this collection.
    type IntoIter: ExactSizeIterator<Item = Self::Owned>;

    /// Returns an interator over items in this collection.
    fn into_iter_owned(self) -> Self::IntoIter;
}

/// Error returned when storage for a collection cannot be reserved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocError;

impl fmt::Display for AllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "collection capacity could not be reserved")
    }
}

impl core::error::Error for AllocError {}

/// An allocatable collection of items using a caller-provided allocator.
pub trait CollectionAllocIn: Collection + Sized {
    /// Allocator used to construct this collection.
    type Alloc: Clone;

    /// Constructs a new, empty collection with at least the specified capacity
    /// using `alloc` and its native infallible failure handling.
    #[must_use]
    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self;

    /// Constructs a collection from `iter` using `alloc` and its native
    /// infallible failure handling.
    #[must_use]
    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, alloc: Self::Alloc) -> Self;

    /// Tries to construct an empty collection with the requested capacity.
    ///
    /// # Errors
    ///
    /// Returns an [`AllocError`] when the requested capacity cannot be
    /// reserved.
    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, AllocError>;

    /// Tries to construct a collection from `iter`.
    ///
    /// # Errors
    ///
    /// Returns an [`AllocError`] when storage for the items cannot be
    /// reserved.
    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, AllocError>;
}

/// An allocatable collection of items.
pub trait CollectionAlloc: Collection + Default + FromIterator<Self::Owned> {
    /// Constructs a new, empty collection with at least the specified capacity.
    fn with_capacity(capacity: usize) -> Self;
}

/// A re-allocatable collection of items.
pub trait CollectionRealloc: CollectionAlloc + Extend<Self::Owned> {
    /// Tries to reserve capacity for at least `additional` more items.
    fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError>;

    /// Tries to extend this collection with the contents of `iter`.
    ///
    /// The collection's logical contents are unchanged when reservation fails.
    fn try_extend<I: IntoIterator<Item = Self::Owned>>(
        &mut self,
        iter: I,
    ) -> Result<(), AllocError>;

    /// Reserves capacity for at least `additional` more items to be inserted in this collection.
    fn reserve(&mut self, additional: usize);

    /// Shortens this collection to `len` items, dropping the rest.
    ///
    /// If `len` is greater than or equal to the current length, this has no
    /// effect.
    fn truncate(&mut self, len: usize);
}

#[cfg(test)]
pub(crate) mod tests {
    extern crate alloc;

    use alloc::vec::Vec;
    use core::fmt::Debug;

    use crate::collection::view::AsView;

    use super::*;

    pub(crate) fn round_trip<
        C: for<'any> CollectionAlloc<Owned = T, View<'any>: Debug>,
        T: for<'this, 'other> AsView<'this, View: Debug> + Clone + Default + Debug + PartialEq,
    >(
        items: impl IntoIterator<Item = T>,
    ) {
        let input = items.into_iter().collect::<Vec<_>>();
        let collection = input.clone().into_iter().collect::<C>();
        let len = collection.len();
        assert_eq!(input.len(), len);
        collection
            .iter_views()
            .enumerate()
            .for_each(|(index, item)| {
                // TODO: compare views
                assert_eq!(input[index], item.into_owned());
            });
        let collection_into_iter = collection.into_iter_owned();
        assert_eq!(collection_into_iter.size_hint(), (len, Some(len)));
        assert_eq!(collection_into_iter.len(), len);
        assert_eq!(input, collection_into_iter.collect::<Vec<_>>());
    }
}
