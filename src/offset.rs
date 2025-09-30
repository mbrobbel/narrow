use std::{
    fmt::{self, Debug},
    iter::{self, Map, Repeat, Zip},
    mem,
    num::TryFromIntError,
    ops::Range,
};

use crate::{
    buffer::{Buffer, VecBuffer},
    collection::{Collection, owned::IntoOwned},
    fixed_size::FixedSize,
    length::Length,
};

pub trait Offset:
    Default + FixedSize + TryInto<usize, Error = TryFromIntError> + sealed::Sealed
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

pub struct Offsets<T: Collection, OffsetItem: Offset = i32, Storage: Buffer = VecBuffer> {
    data: T,
    offsets: Storage::For<OffsetItem>,
}

impl<T: Collection + Debug, OffsetItem: Offset, Storage: Buffer<For<OffsetItem>: Debug>> Debug
    for Offsets<T, OffsetItem, Storage>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Offset")
            .field("data", &self.data)
            .field("offsets", &self.offsets)
            .finish()
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer> Default for Offsets<T, OffsetItem, Storage>
where
    T: Default,
    Storage::For<OffsetItem>: Default + Extend<OffsetItem>,
{
    fn default() -> Self {
        let mut offsets = Storage::For::<OffsetItem>::default();
        offsets.extend(iter::once(OffsetItem::default()));
        Self {
            data: T::default(),
            offsets,
        }
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer> Length
    for Offsets<T, OffsetItem, Storage>
{
    fn len(&self) -> usize {
        self.offsets
            .len()
            .checked_sub(1)
            .expect("offset len underflow")
    }
}

#[expect(missing_debug_implementations)]
pub struct OffsetIntoIter<T: Collection, OffsetItem: Offset, Storage: Buffer> {
    data: T,
    offsets: <Storage::For<OffsetItem> as Collection>::IntoIter,
    position: OffsetItem,
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer> Iterator
    for OffsetIntoIter<T, OffsetItem, Storage>
{
    type Item = Vec<T::Owned>;

    fn next(&mut self) -> Option<Self::Item> {
        let end = self.offsets.next()?;
        let start = mem::replace(&mut self.position, end);
        Some(
            (start.as_usize()..end.as_usize())
                .map(|index| self.data.view(index).expect("out of bounds"))
                .map(IntoOwned::into_owned)
                .collect::<Vec<T::Owned>>(),
        )
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer> Collection
    for Offsets<T, OffsetItem, Storage>
{
    type View<'collection>
        = OffsetView<'collection, T, OffsetItem, Storage>
    where
        Self: 'collection;

    type Owned = Vec<T::Owned>;

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
        Zip<Range<usize>, Repeat<&'collection Self>>,
        fn((usize, &'collection Self)) -> Self::View<'collection>,
    >
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        let len = self.len();
        (0..len)
            .zip(iter::repeat(self))
            .map(|(index, offsets)| offsets.view(index).expect("index in range"))
    }

    type IntoIter = OffsetIntoIter<T, OffsetItem, Storage>;

    fn into_iter_owned(self) -> Self::IntoIter {
        let Self { data, offsets } = self;
        let mut iter = offsets.into_iter_owned();
        let position = iter
            .next()
            .expect("offset buffer must have at least one value");
        OffsetIntoIter {
            data,
            offsets: iter,
            position,
        }
    }
}

#[expect(missing_debug_implementations)]
pub struct OffsetView<'collection, T: Collection, OffsetItem: Offset, Storage: Buffer> {
    collection: &'collection Offsets<T, OffsetItem, Storage>,
    start: usize,
    end: usize,
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer> Clone
    for OffsetView<'_, T, OffsetItem, Storage>
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Collection, OffsetItem: Offset, Storage: Buffer> Copy
    for OffsetView<'_, T, OffsetItem, Storage>
{
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer> IntoOwned<Vec<<T as Collection>::Owned>>
    for OffsetView<'_, T, OffsetItem, Storage>
{
    fn into_owned(self) -> Vec<<T as Collection>::Owned> {
        self.into_iter_owned().collect()
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer> Length
    for OffsetView<'_, T, OffsetItem, Storage>
{
    fn len(&self) -> usize {
        self.end - self.start
    }
}

impl<T: Collection, OffsetItem: Offset, Storage: Buffer> Collection
    for OffsetView<'_, T, OffsetItem, Storage>
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
        Zip<Range<usize>, Repeat<&'collection Self>>,
        fn((usize, &'collection Self)) -> Self::View<'collection>,
    >
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        (0..self.len())
            .zip(iter::repeat(self))
            .map(|(index, collection)| collection.view(index).expect("index in range"))
    }

    type IntoIter = Map<Zip<Range<usize>, Repeat<Self>>, fn((usize, Self)) -> Self::Owned>;

    fn into_iter_owned(self) -> Self::IntoIter {
        (0..self.len())
            .zip(iter::repeat(self))
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
    fn view() {
        let offsets: Offsets<Vec<i32>> = Offsets {
            data: vec![1, 2, 3, 4, 5],
            offsets: vec![0, 2, 3, 4, 5],
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
