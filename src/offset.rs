//! Offset-based collections for storing variable-length items compactly.

extern crate alloc;

use alloc::vec::Vec;

use core::{
    borrow::Borrow,
    fmt::{self, Debug},
    iter::{self, Map, RepeatN, Zip},
    marker::PhantomData,
    mem,
    num::TryFromIntError,
    ops::Range,
};

use crate::{
    buffer::{Buffer, BufferRef, VecBuffer},
    collection::{
        AllocError, ChildRef, Collection, CollectionAlloc, CollectionAllocIn, CollectionRealloc,
        owned::IntoOwned,
    },
    fixed_size::FixedSize,
    length::Length,
};

/// An integer type used to index the values in an [`Offsets`] collection.
///
/// The supported offset types are [`i32`] and [`i64`].
///
/// # Examples
///
/// ```
/// use narrow::offset::Offset;
///
/// fn to_index<T: Offset>(offset: T) -> usize {
///     offset.as_usize()
/// }
/// assert_eq!(to_index(3_i32), 3);
/// ```
pub trait Offset:
    Default
    + FixedSize
    + TryFrom<usize, Error = TryFromIntError>
    + TryInto<usize, Error = TryFromIntError>
    + sealed::Sealed
{
    /// Adds two offsets, panicking if the result overflows.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::offset::Offset;
    ///
    /// assert_eq!(Offset::strict_add(1_i32, 2), 3);
    /// ```
    #[must_use]
    fn strict_add(self, other: Self) -> Self;

    /// Converts this offset to a [`usize`], panicking if it does not fit.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::offset::Offset;
    ///
    /// assert_eq!(3_i64.as_usize(), 3);
    /// ```
    fn as_usize(self) -> usize {
        self.try_into().unwrap_or_else(|e| panic!("{e}"))
    }
}

mod sealed {
    pub trait Sealed {}
    impl<T: super::Offset> Sealed for T {}
}

impl Offset for i32 {
    fn strict_add(self, other: Self) -> Self {
        self.strict_add(other)
    }
}
impl Offset for i64 {
    fn strict_add(self, other: Self) -> Self {
        self.strict_add(other)
    }
}

/// A variable-length collection backed by a flat data collection and offsets.
///
/// The offset buffer contains one more entry than the number of items: each
/// adjacent pair describes the start and end of one item.
///
/// <https://arrow.apache.org/docs/format/Columnar.html#variable-size-binary-layout>
///
/// # Examples
///
/// ```
/// use narrow::{collection::Collection, offset::Offsets};
///
/// let values = [vec![1, 2], vec![3]].into_iter().collect::<Offsets<Vec<i32>>>();
/// assert_eq!(values.owned(0), Some(vec![1, 2]));
/// ```
pub struct Offsets<
    T: Collection,
    OffsetItem: Offset = i32,
    Storage: Buffer = VecBuffer,
    U = Vec<<T as Collection>::Owned>,
> {
    data: T,
    #[allow(clippy::struct_field_names)]
    offsets: Storage::For<OffsetItem>,
    _collection: PhantomData<U>,
}

/// Error returned by [`Offsets::try_from_parts`].
///
/// # Examples
///
/// ```
/// use narrow::offset::{Offsets, OffsetsError};
///
/// let error = Offsets::<Vec<i32>>::try_from_parts(vec![], vec![]).unwrap_err();
/// assert_eq!(error, OffsetsError::Empty);
/// let error = Offsets::<Vec<i32>>::try_from_parts(vec![], vec![1]).unwrap_err();
/// assert_eq!(error, OffsetsError::NonZeroFirst { first: 1 });
/// let error = Offsets::<Vec<i32>>::try_from_parts(vec![], vec![0, -1]).unwrap_err();
/// assert_eq!(error, OffsetsError::Negative { index: 1 });
/// let error = Offsets::<Vec<i32>>::try_from_parts(vec![0], vec![0, 1, 0]).unwrap_err();
/// assert_eq!(error, OffsetsError::NonMonotonic { index: 2 });
/// let error = Offsets::<Vec<i32>>::try_from_parts(vec![], vec![0, 1]).unwrap_err();
/// assert_eq!(error, OffsetsError::OutOfBounds { last: 1, data: 0 });
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OffsetsError {
    /// The offsets buffer is empty; it must contain at least one offset.
    Empty,
    /// The first offset is not zero (required by the Arrow `n + 1`
    /// representation).
    NonZeroFirst {
        /// The value of the first offset.
        first: usize,
    },
    /// The offset at `index` is negative.
    Negative {
        /// The index of the negative offset.
        index: usize,
    },
    /// The offset at `index` is smaller than the preceding offset.
    NonMonotonic {
        /// The index of the offset that breaks monotonicity.
        index: usize,
    },
    /// The last offset exceeds the length of the data.
    OutOfBounds {
        /// The value of the last offset.
        last: usize,
        /// The length of the data.
        data: usize,
    },
}

impl fmt::Display for OffsetsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Empty => write!(f, "offsets buffer must contain at least one offset"),
            Self::NonZeroFirst { first } => write!(f, "first offset ({first}) is not zero"),
            Self::Negative { index } => write!(f, "offset at index {index} is negative"),
            Self::NonMonotonic { index } => {
                write!(f, "offset at index {index} is not monotonically increasing")
            }
            Self::OutOfBounds { last, data } => write!(
                f,
                "last offset ({last}) exceeds the length of the data ({data})"
            ),
        }
    }
}

impl core::error::Error for OffsetsError {}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U> Offsets<T, OffsetItem, Storage, U> {
    /// Constructs [`Offsets`] from a `data` collection and its `offsets`
    /// buffer.
    ///
    /// # Errors
    ///
    /// Returns an [`OffsetsError`] when the offsets buffer is empty, its first
    /// offset is not zero, it contains a negative offset, it is not
    /// monotonically increasing, or when its last offset exceeds the length of
    /// the data.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{collection::Collection, offset::Offsets};
    ///
    /// let values = Offsets::<Vec<i32>>::try_from_parts(vec![1, 2, 3], vec![0, 2, 3]).unwrap();
    /// assert_eq!(values.owned(0), Some(vec![1, 2]));
    /// ```
    pub fn try_from_parts(
        data: T,
        offsets: Storage::For<OffsetItem>,
    ) -> Result<Self, OffsetsError> {
        let mut iter = offsets.borrow().iter().enumerate();
        let (_, first) = iter.next().ok_or(OffsetsError::Empty)?;
        let mut previous: usize = (*first)
            .try_into()
            .map_err(|_| OffsetsError::Negative { index: 0 })?;
        if previous != 0 {
            return Err(OffsetsError::NonZeroFirst { first: previous });
        }
        for (index, offset) in iter {
            let current: usize = (*offset)
                .try_into()
                .map_err(|_| OffsetsError::Negative { index })?;
            if current < previous {
                return Err(OffsetsError::NonMonotonic { index });
            }
            previous = current;
        }
        let data_len = data.len();
        if previous > data_len {
            return Err(OffsetsError::OutOfBounds {
                last: previous,
                data: data_len,
            });
        }
        Ok(Self {
            data,
            offsets,
            _collection: PhantomData,
        })
    }

    /// Returns the data collection and offsets buffer of these [`Offsets`].
    ///
    /// This is the inverse of [`Offsets::try_from_parts`].
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::offset::Offsets;
    ///
    /// let values = Offsets::<Vec<i32>>::try_from_parts(vec![1, 2], vec![0, 2]).unwrap();
    /// assert_eq!(values.into_parts(), (vec![1, 2], vec![0, 2]));
    /// ```
    #[must_use]
    pub fn into_parts(self) -> (T, Storage::For<OffsetItem>) {
        (self.data, self.offsets)
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U> BufferRef
    for Offsets<T, OffsetItem, Storage, U>
{
    type Buffer = Storage::For<OffsetItem>;

    fn buffer_ref(&self) -> &Self::Buffer {
        &self.offsets
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U> ChildRef
    for Offsets<T, OffsetItem, Storage, U>
{
    type Child = T;

    fn child_ref(&self) -> &Self::Child {
        &self.data
    }
}

impl<T: Collection + Debug, OffsetItem: Offset, Storage: Buffer<For<OffsetItem>: Debug>, U> Debug
    for Offsets<T, OffsetItem, Storage, U>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Offset")
            .field("data", &self.data)
            .field("offsets", &self.offsets)
            .finish()
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U> Default
    for Offsets<T, OffsetItem, Storage, U>
where
    T: Default,
    Storage::For<OffsetItem>: Default + CollectionRealloc<Owned = OffsetItem>,
{
    fn default() -> Self {
        let mut offsets = Storage::For::<OffsetItem>::default();
        offsets.extend(iter::once(OffsetItem::default()));
        Self {
            data: T::default(),
            offsets,
            _collection: PhantomData,
        }
    }
}

impl<
    T: CollectionRealloc,
    OffsetItem: Offset,
    Storage: Buffer,
    U: CollectionAlloc<Owned = T::Owned> + FromIterator<T::Owned>,
> CollectionAllocIn for Offsets<T, OffsetItem, Storage, U>
where
    Storage::For<OffsetItem>: CollectionRealloc<Alloc = T::Alloc>,
{
    type Alloc = T::Alloc;

    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self {
        let data = T::with_capacity_in(capacity, alloc.clone());
        let mut offsets =
            Storage::For::<OffsetItem>::with_capacity_in(capacity.strict_add(1), alloc);
        offsets.extend(iter::once(OffsetItem::default()));
        Self {
            data,
            offsets,
            _collection: PhantomData,
        }
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, alloc: Self::Alloc) -> Self {
        let items = iter.into_iter();
        let (lower_bound, upper_bound) = items.size_hint();
        let mut offsets = Self::with_capacity_in(upper_bound.unwrap_or(lower_bound), alloc);
        offsets.extend(items);
        offsets
    }

    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, AllocError> {
        let offset_capacity = capacity.checked_add(1).ok_or(AllocError)?;
        let data = T::try_with_capacity_in(capacity, alloc.clone())?;
        let mut offsets = Storage::For::<OffsetItem>::try_with_capacity_in(offset_capacity, alloc)?;
        offsets.try_extend(iter::once(OffsetItem::default()))?;
        Ok(Self {
            data,
            offsets,
            _collection: PhantomData,
        })
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, AllocError> {
        let items = iter.into_iter();
        let (lower_bound, upper_bound) = items.size_hint();
        let mut offsets = Self::try_with_capacity_in(upper_bound.unwrap_or(lower_bound), alloc)?;
        offsets.try_extend(items)?;
        Ok(offsets)
    }
}

impl<
    T: CollectionRealloc,
    OffsetItem: Offset,
    Storage: Buffer,
    U: CollectionAlloc<Owned = T::Owned>,
> Extend<U> for Offsets<T, OffsetItem, Storage, U>
where
    Storage::For<OffsetItem>: CollectionRealloc<Owned = OffsetItem>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        let mut position = self
            .offsets
            .borrow()
            .last()
            .copied()
            .expect("at least one value in the offsets buffer");

        // Drop any data beyond the last offset (e.g. an unreferenced suffix)
        // so appended items are addressed correctly.
        self.data.truncate(position.as_usize());

        iter.into_iter().for_each(|collection| {
            let next_position = position.strict_add(collection.len().try_into().expect("overflow"));
            self.data.reserve(collection.len());
            self.data.extend(collection.into_iter_owned());
            self.offsets.extend(iter::once(next_position));
            position = next_position;
        });
    }
}

impl<
    T: CollectionRealloc,
    OffsetItem: Offset,
    Storage: Buffer,
    U: CollectionAlloc<Owned = T::Owned>,
> FromIterator<U> for Offsets<T, OffsetItem, Storage, U>
where
    T: Default,
    Storage::For<OffsetItem>: Default + CollectionRealloc<Owned = OffsetItem>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        let mut offsets = Self::default();
        offsets.extend(iter);
        offsets
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U> Length
    for Offsets<T, OffsetItem, Storage, U>
{
    fn len(&self) -> usize {
        self.offsets.len().strict_sub(1)
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U: FromIterator<T::Owned>> Collection
    for Offsets<T, OffsetItem, Storage, U>
{
    type View<'collection>
        = OffsetView<'collection, T, OffsetItem, Storage, U>
    where
        Self: 'collection;

    type Owned = U;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        let start = self.offsets.owned(index);
        let end = self.offsets.owned(index.strict_add(1));
        start.zip(end).map(|(s, e)| OffsetView {
            collection: self,
            start: s.as_usize(),
            end: e.as_usize(),
        })
    }

    type Iter<'collection>
        = Map<
        Zip<Range<usize>, RepeatN<&'collection Self>>,
        fn((usize, &'collection Self)) -> Self::View<'collection>,
    >
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        let len = self.len();
        (0..len)
            .zip(iter::repeat_n(self, len))
            .map(|(index, offsets)| offsets.view(index).expect("index in range"))
    }

    type IntoIter = OffsetIntoIter<T, OffsetItem, Storage, U>;

    fn into_iter_owned(self) -> Self::IntoIter {
        let Self { data, offsets, .. } = self;
        let mut iter = offsets.into_iter_owned();
        let position = iter
            .next()
            .expect("offset buffer must have at least one value");
        OffsetIntoIter {
            data,
            offsets: iter,
            position,
            _collection: PhantomData,
        }
    }
}

impl<
    T: CollectionRealloc,
    OffsetItem: Offset,
    Storage: Buffer,
    U: CollectionAlloc<Owned = T::Owned> + FromIterator<T::Owned>,
> CollectionRealloc for Offsets<T, OffsetItem, Storage, U>
where
    Storage::For<OffsetItem>: CollectionRealloc<Alloc = T::Alloc>,
{
    fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError> {
        // This is only enough for collections with len 1
        self.data.try_reserve(additional)?;
        self.offsets.try_reserve(additional)
    }

    fn try_extend<I: IntoIterator<Item = Self::Owned>>(
        &mut self,
        iter: I,
    ) -> Result<(), AllocError> {
        let rows = self.len();
        let mut position = self
            .offsets
            .borrow()
            .last()
            .copied()
            .expect("at least one value in the offsets buffer");
        let data_len = position.as_usize();
        self.data.truncate(data_len);

        for collection in iter {
            let next_result = position
                .as_usize()
                .checked_add(collection.len())
                .and_then(|value| OffsetItem::try_from(value).ok())
                .ok_or(AllocError);
            let Ok(next_position) = next_result else {
                self.offsets.truncate(rows.strict_add(1));
                self.data.truncate(data_len);
                return Err(AllocError);
            };
            if let Err(error) = self.data.try_extend(collection.into_iter_owned()) {
                self.offsets.truncate(rows.strict_add(1));
                self.data.truncate(data_len);
                return Err(error);
            }
            if let Err(error) = self.offsets.try_extend(iter::once(next_position)) {
                self.offsets.truncate(rows.strict_add(1));
                self.data.truncate(data_len);
                return Err(error);
            }
            position = next_position;
        }
        Ok(())
    }

    fn reserve(&mut self, additional: usize) {
        // This is only enough for collections with len 1
        self.data.reserve(additional);
        self.offsets.reserve(additional);
    }

    fn truncate(&mut self, len: usize) {
        if len < self.len() {
            // Keep `len` lists: `len + 1` offsets and the data they reference.
            let data_len = self.offsets.owned(len).expect("offset in range").as_usize();
            self.offsets.truncate(len.strict_add(1));
            self.data.truncate(data_len);
        }
    }
}

#[expect(missing_debug_implementations)]
/// An owning iterator over the items in an [`Offsets`] collection.
///
/// # Examples
///
/// ```
/// use narrow::{buffer::VecBuffer, collection::Collection, offset::{OffsetIntoIter, Offsets}};
///
/// let values = [vec![1, 2]].into_iter().collect::<Offsets<Vec<i32>>>();
/// let iter: OffsetIntoIter<Vec<i32>, i32, VecBuffer, Vec<i32>> = values.into_iter_owned();
/// assert_eq!(iter.len(), 1);
/// assert_eq!(iter.collect::<Vec<_>>(), [vec![1, 2]]);
/// ```
pub struct OffsetIntoIter<T: Collection, OffsetItem: Offset, Storage: Buffer, U> {
    data: T,
    offsets: <Storage::For<OffsetItem> as Collection>::IntoIter,
    position: OffsetItem,
    _collection: PhantomData<U>,
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U: FromIterator<T::Owned>> Iterator
    for OffsetIntoIter<T, OffsetItem, Storage, U>
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        let end = self.offsets.next()?;
        let start = mem::replace(&mut self.position, end);
        Some(
            (start.as_usize()..end.as_usize())
                .map(|index| self.data.view(index).expect("out of bounds"))
                .map(IntoOwned::into_owned)
                .collect::<U>(),
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U: FromIterator<T::Owned>>
    ExactSizeIterator for OffsetIntoIter<T, OffsetItem, Storage, U>
{
    fn len(&self) -> usize {
        self.offsets.len()
    }
}

/// A borrowed view of one item in an [`Offsets`] collection.
///
/// The view is represented by a range into the collection's flat data and
/// remains valid only while the collection is borrowed.
///
/// # Examples
///
/// ```
/// use narrow::{buffer::VecBuffer, collection::{Collection, owned::IntoOwned}, length::Length, offset::{OffsetView, Offsets}};
///
/// let values = [vec![1, 2]].into_iter().collect::<Offsets<Vec<i32>>>();
/// let view: OffsetView<'_, Vec<i32>, i32, VecBuffer, Vec<i32>> = values.view(0).unwrap();
/// assert_eq!(view.len(), 2);
/// assert!(<_ as PartialEq<Vec<_>>>::eq(&view, &vec![1, 2]));
/// assert_eq!(view.into_owned(), vec![1, 2]);
/// ```
pub struct OffsetView<'collection, T: Collection, OffsetItem: Offset, Storage: Buffer, U> {
    collection: &'collection Offsets<T, OffsetItem, Storage, U>,
    start: usize,
    end: usize,
}

impl<T: for<'any> Collection<View<'any>: Debug>, OffsetItem: Offset, Storage: Buffer, U> Debug
    for OffsetView<'_, T, OffsetItem, Storage, U>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OffsetView")
            .field("offset", &(self.start..self.end))
            .finish_non_exhaustive()
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U> Clone
    for OffsetView<'_, T, OffsetItem, Storage, U>
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U> Copy
    for OffsetView<'_, T, OffsetItem, Storage, U>
{
}

impl<C: Collection, T: Collection, OffsetItem: Offset, Storage: Buffer, U> PartialEq<C>
    for OffsetView<'_, T, OffsetItem, Storage, U>
where
    for<'any, 'other> T::View<'any>: PartialEq<C::View<'other>>,
{
    fn eq(&self, other: &C) -> bool {
        self.len() == other.len()
            && self
                .iter_views()
                .zip(other.iter_views())
                .all(|(a, b)| a == b)
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U: FromIterator<T::Owned>> IntoOwned<U>
    for OffsetView<'_, T, OffsetItem, Storage, U>
{
    fn into_owned(self) -> U {
        self.into_iter_owned().collect()
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U> Length
    for OffsetView<'_, T, OffsetItem, Storage, U>
{
    fn len(&self) -> usize {
        self.end.strict_sub(self.start)
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U> Collection
    for OffsetView<'_, T, OffsetItem, Storage, U>
{
    type View<'collection>
        = <T as Collection>::View<'collection>
    where
        Self: 'collection;

    type Owned = T::Owned;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        let idx = self.start.strict_add(index);
        if idx < self.end {
            self.collection.data.view(idx)
        } else {
            None
        }
    }

    type Iter<'collection>
        = Map<
        Zip<Range<usize>, RepeatN<&'collection Self>>,
        fn((usize, &'collection Self)) -> Self::View<'collection>,
    >
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        (0..self.len())
            .zip(iter::repeat_n(self, self.len()))
            .map(|(index, collection)| collection.view(index).expect("index in range"))
    }

    type IntoIter = Map<Zip<Range<usize>, RepeatN<Self>>, fn((usize, Self)) -> Self::Owned>;

    fn into_iter_owned(self) -> Self::IntoIter {
        (0..self.len())
            .zip(iter::repeat_n(self, self.len()))
            .map(|(index, collection)| collection.view(index).expect("index in range").into_owned())
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec;

    use super::*;

    #[test]
    fn default_len() {
        let offsets: Offsets<Vec<u8>> = Offsets::default();
        assert_eq!(offsets.len(), 0);
        assert_eq!(offsets.data.len(), 0);
        assert_eq!(offsets.offsets.len(), 1);
        assert_eq!(offsets.into_iter_owned().len(), 0);
    }

    #[test]
    fn iterator_size() {
        let a = [vec![42]]
            .into_iter()
            .collect::<Offsets<Vec<_>>>()
            .into_iter_owned();
        assert_eq!(a.size_hint(), (1, Some(1)));
        assert_eq!(a.len(), 1);

        let b = [vec![42, 1]]
            .into_iter()
            .collect::<Offsets<Vec<_>>>()
            .into_iter_owned();
        assert_eq!(b.size_hint(), (1, Some(1)));
        assert_eq!(b.len(), 1);

        let c = [vec![42, 1], vec![2]]
            .into_iter()
            .collect::<Offsets<Vec<_>>>()
            .into_iter_owned();
        assert_eq!(c.size_hint(), (2, Some(2)));
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn compare_offset_view() {
        let collection = [vec![42], vec![0]].into_iter().collect();
        let view: OffsetView<Vec<u8>, i32, VecBuffer, Vec<u8>> = OffsetView {
            collection: &collection,
            start: 0,
            end: 1,
        };
        assert!(<_ as PartialEq<Vec<_>>>::eq(&view, &vec![42]));
    }

    #[test]
    fn try_from_parts() {
        let offsets = Offsets::<Vec<i32>>::try_from_parts(vec![1, 2, 3, 4, 5], vec![0, 2, 3, 4, 5])
            .expect("valid parts");
        assert_eq!(offsets.len(), 4);
        let (data, offsets_buffer) = offsets.into_parts();
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
        assert_eq!(offsets_buffer, vec![0, 2, 3, 4, 5]);
    }

    #[test]
    fn try_from_parts_empty() {
        let error =
            Offsets::<Vec<i32>>::try_from_parts(vec![1], vec![]).expect_err("empty offsets");
        assert_eq!(error, OffsetsError::Empty);
    }

    #[test]
    fn try_from_parts_non_zero_first() {
        let error = Offsets::<Vec<i32>>::try_from_parts(vec![1, 2], vec![1, 2])
            .expect_err("non-zero first offset");
        assert_eq!(error, OffsetsError::NonZeroFirst { first: 1 });
    }

    #[test]
    fn try_from_parts_negative() {
        let error =
            Offsets::<Vec<i32>>::try_from_parts(vec![1, 2], vec![-1, 0]).expect_err("negative");
        assert_eq!(error, OffsetsError::Negative { index: 0 });
    }

    #[test]
    fn try_from_parts_non_monotonic() {
        let error = Offsets::<Vec<i32>>::try_from_parts(vec![1, 2, 3], vec![0, 2, 1])
            .expect_err("non-monotonic");
        assert_eq!(error, OffsetsError::NonMonotonic { index: 2 });
    }

    #[test]
    fn try_from_parts_out_of_bounds() {
        let error =
            Offsets::<Vec<i32>>::try_from_parts(vec![1, 2], vec![0, 5]).expect_err("out of bounds");
        assert_eq!(error, OffsetsError::OutOfBounds { last: 5, data: 2 });
    }

    #[test]
    fn truncate() {
        let mut offsets = [vec![1, 2], vec![3], vec![4, 5, 6]]
            .into_iter()
            .collect::<Offsets<Vec<i32>>>();
        assert_eq!(offsets.len(), 3);
        offsets.truncate(1);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets.owned(0), Some(vec![1, 2]));
        assert_eq!(offsets.owned(1), None);
    }

    #[test]
    fn extend_reconciles_trailing_data() {
        // Offsets referencing only the first list, with an unreferenced
        // trailing element (`99`) in the data buffer.
        let mut offsets: Offsets<Vec<i32>> = Offsets {
            data: vec![1, 2, 99],
            offsets: vec![0, 2],
            _collection: PhantomData,
        };
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets.owned(0), Some(vec![1, 2]));
        // Appending must overwrite the unreferenced trailing data, not append
        // past it.
        offsets.extend([vec![3]]);
        assert_eq!(offsets.len(), 2);
        assert_eq!(offsets.owned(1), Some(vec![3]));
    }

    #[test]
    fn view() {
        let offsets: Offsets<Vec<i32>> = Offsets {
            data: vec![1, 2, 3, 4, 5],
            offsets: vec![0, 2, 3, 4, 5],
            _collection: PhantomData,
        };

        assert_eq!(offsets.len(), 4);

        let slice = offsets.view(0).expect("a value");
        assert_eq!(slice.start, 0);
        assert_eq!(slice.end, 2);
        assert_eq!(slice.view(0), Some(1));
        assert_eq!(slice.view(1), Some(2));
        assert_eq!(slice.view(2), None);

        let views = offsets.iter_views().collect::<Vec<_>>();
        assert_eq!(views[0].into_owned(), vec![1, 2]);
        assert_eq!(views[1].into_iter_owned().collect::<Vec<_>>(), vec![3]);
        assert_eq!(views[2].iter_views().collect::<Vec<_>>(), vec![4]);
    }
}
