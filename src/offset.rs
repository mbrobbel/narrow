use std::{
    borrow::Borrow,
    fmt::{self, Debug},
    iter::{self, Map, RepeatN, Zip},
    marker::PhantomData,
    mem,
    num::TryFromIntError,
    ops::{AddAssign, Range},
};

use crate::{
    buffer::{Buffer, VecBuffer},
    collection::{Collection, CollectionAlloc, CollectionRealloc, owned::IntoOwned},
    fixed_size::FixedSize,
    length::Length,
};

// todo: checked add
pub trait Offset:
    AddAssign<Self>
    + Default
    + FixedSize
    + TryFrom<usize, Error = TryFromIntError>
    + TryInto<usize, Error = TryFromIntError>
    + sealed::Sealed
{
    fn as_usize(self) -> usize {
        self.try_into().unwrap_or_else(|e| panic!("{e}"))
    }
}

mod sealed {
    pub trait Sealed {}
    impl<T: super::Offset> Sealed for T {}
}

impl Offset for i32 {}
impl Offset for i64 {}

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
    Storage::For<OffsetItem>: CollectionRealloc<Owned = OffsetItem>,
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

        iter.into_iter().for_each(|collection| {
            position += collection.len().try_into().expect("overflow");
            self.offsets.extend(iter::once(position));
            self.data.reserve(collection.len());
            for item in collection.into_iter_owned() {
                self.data.extend(std::iter::once(item));
            }
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
    Storage::For<OffsetItem>: CollectionRealloc<Owned = OffsetItem>,
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
        self.offsets
            .len()
            .checked_sub(1)
            .expect("offset len underflow")
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
        let end = self.offsets.owned(index + 1);
        start.zip(end).map(|(start, end)| OffsetView {
            collection: self,
            start: start.as_usize(),
            end: end.as_usize(),
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
    U: CollectionAlloc<Owned = T::Owned>,
> CollectionAlloc for Offsets<T, OffsetItem, Storage, U>
where
    Storage::For<OffsetItem>: CollectionRealloc<Owned = OffsetItem>,
{
    fn with_capacity(capacity: usize) -> Self {
        Self {
            data: T::with_capacity(capacity),
            offsets: Storage::For::<OffsetItem>::with_capacity(capacity),
            _collection: PhantomData,
        }
    }
}

impl<
    T: CollectionRealloc,
    OffsetItem: Offset,
    Storage: Buffer,
    U: CollectionAlloc<Owned = T::Owned>,
> CollectionRealloc for Offsets<T, OffsetItem, Storage, U>
where
    Storage::For<OffsetItem>: CollectionRealloc<Owned = OffsetItem>,
{
    fn reserve(&mut self, additional: usize) {
        // This is only enough for collections with len 1
        self.data.reserve(additional);
        self.offsets.reserve(additional);
    }
}

#[expect(missing_debug_implementations)]
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
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer, U: FromIterator<T::Owned>>
    ExactSizeIterator for OffsetIntoIter<T, OffsetItem, Storage, U>
{
    fn len(&self) -> usize {
        self.offsets.len()
    }
}

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
        self.end - self.start
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
        let idx = self.start.checked_add(index).expect("overflow");
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
    use super::*;

    #[test]
    fn default_len() {
        let offsets: Offsets<Vec<u8>> = Offsets::default();
        assert_eq!(offsets.len(), 0);
        assert_eq!(offsets.data.len(), 0);
        assert_eq!(offsets.offsets.len(), 1);
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
