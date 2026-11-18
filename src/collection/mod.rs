//! Collections of items.

pub mod owned;
pub mod view;

#[cfg(target_has_atomic = "ptr")]
pub mod arc;
pub mod array;
pub mod r#box;
pub mod rc;
pub mod slice;
pub mod vec;

pub mod flatten;

use core::{alloc::AllocError, fmt, ops::Range};

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
    ///
    /// It knows its exact length, so iterators built on it, such as those of
    /// variable-size list items, can as well.
    type Iter<'collection>: ExactSizeIterator<Item = Self::View<'collection>>
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

    /// Returns an iterator over references to the items at the indices in
    /// `range`.
    ///
    /// Indices beyond the length of this collection are ignored, so a range
    /// that starts past the end, or past its own end, yields nothing.
    ///
    /// Nested collections read the items of one row through it, which lets
    /// implementations iterate a contiguous part of their storage directly
    /// instead of looking up one index at a time.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::Collection;
    ///
    /// let values = [1, 2, 3, 4];
    /// assert_eq!(values.iter_views_range(1..3).collect::<Vec<_>>(), [2, 3]);
    /// assert_eq!(values.iter_views_range(3..10).collect::<Vec<_>>(), [4]);
    /// assert_eq!(values.iter_views_range(5..10).len(), 0);
    /// ```
    fn iter_views_range(&self, range: Range<usize>) -> Self::Iter<'_>;

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

/// Returns `range` restricted to the indices below `len`.
///
/// A range that starts past `len`, or past its own end, becomes empty.
pub(crate) fn clamp_range(range: Range<usize>, len: usize) -> Range<usize> {
    let end = range.end.min(len);
    range.start.min(end)..end
}

/// Immutable access to a physical child collection.
///
/// A physical child does not necessarily correspond to an Arrow schema child.
/// For example, variable-size binary data exposes a physical child even though
/// Arrow represents it as a data buffer.
///
/// Nested layouts are wrappers around other physical collections. A shared
/// child-access trait lets exporters and introspection code recurse through
/// those wrappers without matching every concrete layout, while the name
/// deliberately distinguishes physical children from schema children.
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

/// Error returned when a collection cannot reserve capacity.
///
/// Like `alloc`'s [`TryReserveError`][std], it tells apart a request the
/// collection cannot represent from an allocator that failed to provide the
/// memory.
///
/// [std]: https://doc.rust-lang.org/alloc/collections/struct.TryReserveError.html
///
/// # Examples
///
/// ```
/// use narrow::collection::{CollectionAllocIn, TryReserveError};
///
/// let result = Vec::<u8>::try_with_capacity_in(usize::MAX, ());
/// assert_eq!(result, Err(TryReserveError::CapacityOverflow));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryReserveError {
    /// The requested capacity exceeds what the collection can address, such
    /// as more than `isize::MAX` bytes or more items than its offsets can
    /// represent.
    CapacityOverflow,
    /// The allocator failed to provide the memory.
    AllocError(AllocError),
}

impl From<AllocError> for TryReserveError {
    fn from(error: AllocError) -> Self {
        Self::AllocError(error)
    }
}

impl fmt::Display for TryReserveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::CapacityOverflow => {
                f.write_str("capacity exceeds what the collection can address")
            }
            Self::AllocError(_) => f.write_str("memory allocation failed"),
        }
    }
}

impl core::error::Error for TryReserveError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match *self {
            Self::CapacityOverflow => None,
            Self::AllocError(ref error) => Some(error),
        }
    }
}

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
    /// let values = <Vec<u8> as CollectionAllocIn>::with_capacity_in(4, ());
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
    /// Returns [`TryReserveError::CapacityOverflow`] when `capacity` exceeds
    /// what the collection can address, or [`TryReserveError::AllocError`]
    /// when the allocator fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::CollectionAllocIn;
    ///
    /// assert!(Vec::<u8>::try_with_capacity_in(4, ()).is_ok());
    /// ```
    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, TryReserveError>;

    /// Tries to construct a collection from `iter`.
    ///
    /// # Errors
    ///
    /// Returns [`TryReserveError::CapacityOverflow`] when the items exceed what
    /// the collection can address, or [`TryReserveError::AllocError`] when the
    /// allocator fails.
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
    ) -> Result<Self, TryReserveError>;
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
    /// Returns the allocator used by this collection.
    ///
    /// Composite collections use the cloned handle when they need to create a
    /// child buffer after construction.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::CollectionRealloc;
    ///
    /// let values = vec![1_u8];
    /// assert_eq!(CollectionRealloc::allocator(&values), ());
    /// ```
    fn allocator(&self) -> Self::Alloc;

    /// Tries to reserve capacity for at least `additional` more items.
    ///
    /// # Errors
    ///
    /// Returns [`TryReserveError::CapacityOverflow`] when the new capacity
    /// exceeds what the collection can address, or
    /// [`TryReserveError::AllocError`] when the allocator fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::CollectionRealloc;
    ///
    /// let mut values = Vec::<u8>::new();
    /// assert!(CollectionRealloc::try_reserve(&mut values, 4).is_ok());
    /// ```
    fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError>;

    /// Tries to extend this collection with the contents of `iter`.
    ///
    /// # Errors
    ///
    /// Returns [`TryReserveError::CapacityOverflow`] when the extended
    /// collection would exceed what it can address, or
    /// [`TryReserveError::AllocError`] when the allocator fails. The
    /// collection's logical contents are unchanged when this fails.
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
    ) -> Result<(), TryReserveError>;

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

    use alloc::{vec, vec::Vec};
    use core::fmt::Debug;

    use crate::collection::view::AsView;

    use super::*;

    fn assert_collection_alloc<C: CollectionAlloc>() {}

    #[test]
    fn try_reserve_error_source() {
        use alloc::string::ToString;
        use core::error::Error;

        let overflow = TryReserveError::CapacityOverflow;
        assert!(overflow.source().is_none());
        assert_eq!(
            overflow.to_string(),
            "capacity exceeds what the collection can address"
        );

        let failed = TryReserveError::from(AllocError);
        assert_eq!(failed, TryReserveError::AllocError(AllocError));
        assert!(failed.source().is_some_and(<dyn Error>::is::<AllocError>));
        assert_eq!(failed.to_string(), "memory allocation failed");
    }

    #[test]
    fn collection_alloc_is_blanket_implemented() {
        assert_collection_alloc::<Vec<u32>>();
    }

    /// The ranges checked against each collection of length `len`: every
    /// range up to two past the end, including reversed ones, and ranges
    /// that end or start at `usize::MAX`.
    fn ranges(len: usize) -> impl Iterator<Item = Range<usize>> {
        let bounds = 0..=len.strict_add(2);
        bounds
            .clone()
            .flat_map(move |start| bounds.clone().map(move |end| start..end))
            .chain((0..=len.strict_add(1)).map(|start| start..usize::MAX))
            .chain([
                usize::MAX..usize::MAX,
                Range {
                    start: usize::MAX,
                    end: 0,
                },
            ])
    }

    /// Asserts that [`Collection::iter_views_range`] yields the items that
    /// skipping and taking from [`Collection::iter_views`] yields, and knows
    /// how many.
    pub(crate) fn assert_iter_views_range<'collection, C: Collection<Owned: Debug + PartialEq>>(
        collection: &'collection C,
    ) {
        let owned = |views: C::Iter<'collection>| {
            views.map(IntoOwned::into_owned).collect::<Vec<C::Owned>>()
        };
        for range in ranges(collection.len()) {
            let expected = collection
                .iter_views()
                .skip(range.start)
                .take(range.end.saturating_sub(range.start))
                .map(IntoOwned::into_owned)
                .collect::<Vec<_>>();
            let views = collection.iter_views_range(range.clone());
            assert_eq!(views.len(), expected.len(), "{range:?}");
            assert_eq!(
                views.size_hint(),
                (expected.len(), Some(expected.len())),
                "{range:?}"
            );
            let mut stepped = collection.iter_views_range(range.clone());
            assert_eq!(
                stepped.next().map(IntoOwned::into_owned).as_ref(),
                expected.first(),
                "{range:?}"
            );
            assert_eq!(stepped.len(), expected.len().saturating_sub(1), "{range:?}");
            assert_eq!(owned(views), expected, "{range:?}");
        }
    }

    /// Like [`assert_iter_views_range`], and also asserts that iterating the
    /// range from the back yields its items in reverse.
    pub(crate) fn assert_iter_views_range_rev<
        'collection,
        C: Collection<Owned: Debug + PartialEq, Iter<'collection>: DoubleEndedIterator>,
    >(
        collection: &'collection C,
    ) {
        assert_iter_views_range(collection);
        for range in ranges(collection.len()) {
            let mut expected = collection
                .iter_views_range(range.clone())
                .map(IntoOwned::into_owned)
                .collect::<Vec<_>>();
            expected.reverse();
            let mut views = collection.iter_views_range(range.clone());
            assert_eq!(
                views.next_back().map(IntoOwned::into_owned).as_ref(),
                expected.first(),
                "{range:?}"
            );
            assert_eq!(views.len(), expected.len().saturating_sub(1), "{range:?}");
            assert_eq!(
                views.rev().map(IntoOwned::into_owned).collect::<Vec<_>>(),
                expected.get(1..).unwrap_or_default(),
                "{range:?}"
            );
        }
    }

    #[test]
    fn iter_views_range_slices() {
        let values = [1, 2, 3, 4, 5];
        assert_iter_views_range_rev(&values);
        assert_iter_views_range_rev(&values.as_slice());
        assert_iter_views_range_rev(&values.to_vec());
        assert_iter_views_range_rev(&Vec::<i32>::new());
        assert_iter_views_range_rev(&alloc::boxed::Box::<[i32]>::from(values));
        assert_iter_views_range_rev(&alloc::rc::Rc::<[i32]>::from(values));
        #[cfg(target_has_atomic = "ptr")]
        assert_iter_views_range_rev(&alloc::sync::Arc::<[i32]>::from(values));
        assert_iter_views_range_rev(&[Some(1), None, Some(3)].to_vec());
        assert_iter_views_range_rev(&[alloc::vec![1], alloc::vec![], alloc::vec![2, 3]]);
        assert_iter_views_range_rev(&[[1, 2], [3, 4], [5, 6]].to_vec());
        assert_iter_views_range_rev(&[true, false, true].to_vec());
        assert_iter_views_range_rev(&[(), (), ()].to_vec());
        assert_iter_views_range_rev(&[Some(alloc::vec![1]), None, Some(alloc::vec![])].to_vec());
        assert_iter_views_range_rev(&[Some([1, 2]), None, Some([5, 6])].to_vec());
    }

    #[test]
    fn iter_views_range_bitmaps() {
        use crate::{
            bitmap::Bitmap,
            buffer::{SliceBuffer, VecBuffer},
        };

        let bits = [
            true, false, true, true, false, false, true, false, true, true,
        ];
        assert_iter_views_range_rev(&bits.iter().collect::<Bitmap>());
        let bytes = [0b1010_1101, 0b0110_0011, 0b1111_0000];
        for offset in [0, 1, 7, 8, 9] {
            for len in [0, 1, 7, 8, 9, 15] {
                assert_iter_views_range_rev(&Bitmap::<SliceBuffer>::from_parts(
                    &bytes, len, offset,
                ));
            }
        }
        assert_iter_views_range_rev(&Bitmap::<VecBuffer>::default());
    }

    #[test]
    fn iter_views_range_nulls() {
        use crate::layout::null::Nulls;

        assert_iter_views_range_rev(&Nulls::<()>::new(0));
        assert_iter_views_range_rev(&Nulls::<()>::new(5));
    }

    #[test]
    fn iter_views_range_validity() {
        use crate::validity::Validity;

        let nullable = [Some(1), None, Some(3), None, None, Some(6)]
            .into_iter()
            .collect::<Validity<Vec<i32>>>();
        assert_iter_views_range(&nullable);
        let valid = [Some(1), Some(2), Some(3)]
            .into_iter()
            .collect::<Validity<Vec<i32>>>();
        assert_iter_views_range(&valid);
        assert_iter_views_range(&Validity::<Vec<i32>>::from_collection(vec![1, 2, 3]));
        assert_iter_views_range(&Validity::<Vec<i32>>::default());
    }

    #[test]
    fn iter_views_range_flatten() {
        use crate::collection::flatten::Flatten;

        let lists = [[1, 2], [3, 4], [5, 6], [7, 8]]
            .into_iter()
            .collect::<Flatten<Vec<i32>, 2>>();
        assert_iter_views_range_rev(&lists);
        assert_iter_views_range_rev(&Flatten::<Vec<i32>, 3>::default());
    }

    #[test]
    fn iter_views_range_offsets() {
        use crate::offset::Offsets;

        let lists = [vec![1, 2], vec![], vec![3, 4, 5], vec![6], vec![]]
            .into_iter()
            .collect::<Offsets<Vec<i32>>>();
        assert_iter_views_range_rev(&lists);
        lists.iter_views().for_each(|list| {
            assert_iter_views_range_rev(&list);
        });
        assert_iter_views_range_rev(&Offsets::<Vec<i32>>::default());

        let nested = [vec![vec![1], vec![]], vec![], vec![vec![2, 3], vec![4]]]
            .into_iter()
            .collect::<Offsets<Offsets<Vec<i32>>>>();
        assert_iter_views_range_rev(&nested);
        nested.iter_views().for_each(|list| {
            assert_iter_views_range_rev(&list);
        });
    }

    #[test]
    fn iter_views_range_arrays() {
        use crate::array::Array;

        assert_iter_views_range_rev(&[1, 2, 3].into_iter().collect::<Array<i32>>());
        assert_iter_views_range_rev(&[true, false, true].into_iter().collect::<Array<bool>>());
        assert_iter_views_range_rev(&[(), (), ()].into_iter().collect::<Array<()>>());
        assert_iter_views_range_rev(&[[1, 2], [3, 4]].into_iter().collect::<Array<[i32; 2]>>());
        assert_iter_views_range_rev(
            &[vec![1], vec![], vec![2, 3]]
                .into_iter()
                .collect::<Array<Vec<i32>>>(),
        );
        assert_iter_views_range(&[Some(1), None].into_iter().collect::<Array<Option<i32>>>());
        assert_iter_views_range(
            &[Some(true), None, Some(false)]
                .into_iter()
                .collect::<Array<Option<bool>>>(),
        );
        assert_iter_views_range(
            &[Some([1, 2]), None]
                .into_iter()
                .collect::<Array<Option<[i32; 2]>>>(),
        );
        assert_iter_views_range(
            &[Some(vec![1]), None, Some(vec![2, 3])]
                .into_iter()
                .collect::<Array<Option<Vec<i32>>>>(),
        );
    }

    pub(crate) fn round_trip<
        C: for<'any> CollectionAlloc<Owned = T, View<'any>: Debug>,
        T: for<'this> AsView<'this, View: Debug> + Clone + Default + Debug + PartialEq,
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
