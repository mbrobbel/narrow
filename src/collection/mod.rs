//! Collections of items.

pub mod owned;
pub mod view;

pub mod arc;
pub mod array;
pub mod r#box;
pub mod rc;
pub mod slice;
pub mod vec;

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

/// An allocatable collection of items.
pub trait CollectionAlloc: Collection + Default + FromIterator<Self::Owned> {
    /// Constructs a new, empty collection with at least the specified capacity.
    fn with_capacity(capacity: usize) -> Self;
}

/// A re-allocatable collection of items.
pub trait CollectionRealloc: CollectionAlloc + Extend<Self::Owned> {
    /// Reserves capacity for at least `additional` more items to be inserted in this collection.
    fn reserve(&mut self, additional: usize);
}

#[cfg(test)]
pub(crate) mod tests {
    use std::fmt::Debug;

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
        assert_eq!(input.len(), collection.len());
        collection
            .iter_views()
            .enumerate()
            .for_each(|(index, item)| {
                assert_eq!(input[index], item.into_owned());
            });
        assert_eq!(input, collection.into_iter_owned().collect::<Vec<_>>());
    }
}
