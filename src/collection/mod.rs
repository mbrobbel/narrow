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
///
/// Arrow values range from copyable scalars to borrowed slices of nested
/// data. `Collection` gives both forms one physical-sequence interface while
/// allowing each implementation to choose its cheapest view:
///
/// ```text
/// Collection
/// |-- View<'a>  borrowed access
/// `-- Owned     consuming access
/// ```
///
/// # Examples
///
/// ```
/// use narrow::collection::Collection;
///
/// let values = vec![1, 2, 3];
/// assert_eq!(values.view(1), Some(2));
/// assert_eq!(values.iter_views().sum::<i32>(), 6);
/// ```
pub trait Collection: Length {
    /// Borrowed view of an item in this collection
    type View<'collection>: Copy + IntoOwned<Self::Owned> + 'collection
    where
        Self: 'collection;

    /// Owned items in this collection
    type Owned;

    /// Returns a reference to an item at the given index in this collection or
    /// `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::Collection;
    ///
    /// assert_eq!([1, 2].view(1), Some(2));
    /// assert_eq!([1, 2].view(2), None);
    /// ```
    fn view(&self, index: usize) -> Option<Self::View<'_>>;

    /// Returns an owned item at the given index in this collection or `None`
    /// if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::Collection;
    ///
    /// assert_eq!([1, 2].owned(0), Some(1));
    /// ```
    fn owned(&self, index: usize) -> Option<Self::Owned> {
        self.view(index).map(IntoOwned::into_owned)
    }

    /// Iterator over referenced items in this collection.
    type Iter<'collection>: Iterator<Item = Self::View<'collection>>
    where
        Self: 'collection;

    /// Returns an iterator over references to the items in this collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::Collection;
    ///
    /// assert_eq!([1, 2].iter_views().sum::<i32>(), 3);
    /// ```
    fn iter_views(&self) -> Self::Iter<'_>;

    /// Iterator over owned items in this collection.
    type IntoIter: ExactSizeIterator<Item = Self::Owned>;

    /// Returns an iterator over items in this collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::Collection;
    ///
    /// let values: Vec<_> = [1, 2].into_iter_owned().collect();
    /// assert_eq!(values, [1, 2]);
    /// ```
    fn into_iter_owned(self) -> Self::IntoIter;
}

/// Immutable access to a physical child collection.
///
/// A physical child does not necessarily correspond to an Arrow schema child.
/// For example, variable-size binary data is a physical child in Narrow but an
/// Arrow data buffer.
///
/// # Examples
///
/// ```
/// use narrow::{collection::ChildRef, offset::Offsets};
///
/// let values = [vec![1, 2]].into_iter().collect::<Offsets<Vec<i32>>>();
/// assert_eq!(values.child_ref(), &[1, 2]);
/// ```
pub trait ChildRef {
    /// Physical child collection.
    type Child: Collection;

    /// Returns the physical child collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{collection::ChildRef, offset::Offsets};
    ///
    /// let values = [vec![1, 2]].into_iter().collect::<Offsets<Vec<i32>>>();
    /// assert_eq!(values.child_ref(), &[1, 2]);
    /// ```
    fn child_ref(&self) -> &Self::Child;
}

/// Error returned when storage for a collection cannot be reserved.
///
/// A backend-neutral error keeps fallible construction generic over the
/// storage selected by [`Buffer`](crate::buffer::Buffer).
///
/// # Examples
///
/// ```
/// use narrow::collection::{AllocError, CollectionAllocIn};
///
/// let result = Vec::<u8>::try_with_capacity_in(usize::MAX, ());
/// assert_eq!(result, Err(AllocError));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocError;

impl fmt::Display for AllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "collection capacity could not be reserved")
    }
}

impl core::error::Error for AllocError {}

/// An allocatable collection of items using a caller-provided allocator.
///
/// Allocation is a separate capability from [`Collection`], so borrowed and
/// fixed-capacity storage can still represent Arrow data. Making the allocator
/// explicit also lets nested layouts route child allocations through the same
/// storage policy.
///
/// # Examples
///
/// ```
/// use narrow::collection::CollectionAllocIn;
///
/// let values = Vec::<u8>::from_iter_in([1, 2], ());
/// assert_eq!(values, [1, 2]);
/// ```
pub trait CollectionAllocIn: Collection + Sized {
    /// Allocator used to construct this collection.
    type Alloc: Clone;

    /// Constructs a new, empty collection with at least the specified capacity
    /// using `alloc`.
    ///
    /// Unlike [`CollectionAllocIn::try_with_capacity_in`], this method does not
    /// return reservation failures. Implementations use their allocator's
    /// native infallible failure handling.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::CollectionAllocIn;
    ///
    /// let values = Vec::<u8>::with_capacity_in(4, ());
    /// assert!(values.capacity() >= 4);
    /// ```
    #[must_use]
    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self;

    /// Constructs a collection from `iter` using `alloc`.
    ///
    /// Unlike [`CollectionAllocIn::try_from_iter_in`], this method does not
    /// return reservation failures. Implementations use their allocator's
    /// native infallible failure handling.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::CollectionAllocIn;
    ///
    /// assert_eq!(Vec::<u8>::from_iter_in([1, 2], ()), [1, 2]);
    /// ```
    #[must_use]
    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, alloc: Self::Alloc) -> Self;

    /// Tries to construct an empty collection with the requested capacity.
    ///
    /// # Errors
    ///
    /// Returns an [`AllocError`] when the requested capacity cannot be
    /// reserved.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::CollectionAllocIn;
    ///
    /// assert!(Vec::<u8>::try_with_capacity_in(4, ()).is_ok());
    /// ```
    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, AllocError>;

    /// Tries to construct a collection from `iter`.
    ///
    /// # Errors
    ///
    /// Returns an [`AllocError`] when storage for the items cannot be
    /// reserved.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::CollectionAllocIn;
    ///
    /// let values = Vec::<u8>::try_from_iter_in([1, 2], ()).unwrap();
    /// assert_eq!(values, [1, 2]);
    /// ```
    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, AllocError>;
}

/// An allocatable collection of items using its default allocator.
///
/// This trait is implemented automatically for every [`CollectionAllocIn`]
/// whose allocator implements [`Default`] and which can be constructed through
/// [`Default`] and [`FromIterator`].
///
/// This convenience layer removes the allocator argument only when a storage
/// backend has a natural default. Generic layout code can use
/// [`CollectionAllocIn`] when that assumption does not hold.
///
/// # Examples
///
/// ```
/// use narrow::collection::CollectionAlloc;
///
/// let values = <Vec<u8> as CollectionAlloc>::with_capacity(4);
/// assert!(values.capacity() >= 4);
/// ```
pub trait CollectionAlloc:
    CollectionAllocIn<Alloc: Default> + Default + FromIterator<Self::Owned>
{
    /// Constructs a new, empty collection with at least the specified capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::CollectionAlloc;
    ///
    /// let values = <Vec<u8> as CollectionAlloc>::with_capacity(4);
    /// assert!(values.capacity() >= 4);
    /// ```
    #[must_use]
    fn with_capacity(capacity: usize) -> Self {
        <Self as CollectionAllocIn>::with_capacity_in(capacity, Default::default())
    }
}

impl<C> CollectionAlloc for C
where
    C: CollectionAllocIn + Default + FromIterator<C::Owned>,
    C::Alloc: Default,
{
}

/// A re-allocatable collection of items.
///
/// Growth is modeled separately from initial allocation because not every
/// Arrow buffer can grow. Builders can require this trait without excluding
/// borrowed or fixed-capacity collections from read-only APIs.
///
/// # Examples
///
/// ```
/// use narrow::collection::CollectionRealloc;
///
/// let mut values = vec![1];
/// CollectionRealloc::reserve(&mut values, 1);
/// values.extend([2]);
/// assert_eq!(values, [1, 2]);
/// ```
pub trait CollectionRealloc: CollectionAllocIn + Extend<Self::Owned> {
    /// Tries to reserve capacity for at least `additional` more items.
    ///
    /// # Errors
    ///
    /// Returns an [`AllocError`] when the requested capacity cannot be
    /// reserved.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::CollectionRealloc;
    ///
    /// let mut values = Vec::<u8>::new();
    /// assert!(CollectionRealloc::try_reserve(&mut values, 4).is_ok());
    /// ```
    fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError>;

    /// Tries to extend this collection with the contents of `iter`.
    ///
    /// # Errors
    ///
    /// Returns an [`AllocError`] when storage for the additional items cannot
    /// be reserved. The collection's logical contents are unchanged when a
    /// reservation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::CollectionRealloc;
    ///
    /// let mut values = vec![1];
    /// CollectionRealloc::try_extend(&mut values, [2]).unwrap();
    /// assert_eq!(values, [1, 2]);
    /// ```
    fn try_extend<I: IntoIterator<Item = Self::Owned>>(
        &mut self,
        iter: I,
    ) -> Result<(), AllocError>;

    /// Reserves capacity for at least `additional` more items to be inserted in this collection.
    ///
    /// Unlike [`CollectionRealloc::try_reserve`], this method does not return
    /// reservation failures. Implementations use their allocator's native
    /// infallible failure handling.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::CollectionRealloc;
    ///
    /// let mut values = Vec::<u8>::new();
    /// CollectionRealloc::reserve(&mut values, 4);
    /// assert!(values.capacity() >= 4);
    /// ```
    fn reserve(&mut self, additional: usize);

    /// Shortens this collection to `len` items, dropping the rest.
    ///
    /// If `len` is greater than or equal to the current length, this has no
    /// effect.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::CollectionRealloc;
    ///
    /// let mut values = vec![1, 2, 3];
    /// CollectionRealloc::truncate(&mut values, 2);
    /// assert_eq!(values, [1, 2]);
    /// CollectionRealloc::truncate(&mut values, 4);
    /// assert_eq!(values, [1, 2]);
    /// ```
    fn truncate(&mut self, len: usize);
}

#[cfg(test)]
pub(crate) mod tests {
    extern crate alloc;

    use alloc::vec::Vec;
    use core::fmt::Debug;

    use crate::collection::view::AsView;

    use super::*;

    fn assert_collection_alloc<C: CollectionAlloc>() {}

    #[test]
    fn collection_alloc_is_blanket_implemented() {
        assert_collection_alloc::<Vec<u32>>();
    }

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
