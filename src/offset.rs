use std::{
    fmt::{self, Debug},
    iter::{self, Map, Repeat, Zip},
    mem,
    num::TryFromIntError,
    ops::Range,
};

use crate::{
    buffer::{BufferType, VecBuffer},
    collection::{Collection, IntoOwned},
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

pub struct Offsets<T: Collection, OffsetItem: Offset = i32, Buffer: BufferType = VecBuffer> {
    data: T,
    offsets: Buffer::Buffer<OffsetItem>,
}

impl<T: Collection, OffsetItem: Offset, Buffer: BufferType> Debug for Offsets<T, OffsetItem, Buffer>
where
    T: Debug,
    Buffer::Buffer<OffsetItem>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Offset")
            .field("data", &self.data)
            .field("offsets", &self.offsets)
            .finish()
    }
}

impl<T: Collection, OffsetItem: Offset, Buffer: BufferType> Default
    for Offsets<T, OffsetItem, Buffer>
where
    T: Default,
    Buffer::Buffer<OffsetItem>: Default + Extend<OffsetItem>,
{
    fn default() -> Self {
        let mut offsets = Buffer::Buffer::<OffsetItem>::default();
        offsets.extend(iter::once(OffsetItem::default()));
        Self {
            data: T::default(),
            offsets,
        }
    }
}

impl<T: Collection, OffsetItem: Offset, Buffer: BufferType> Length
    for Offsets<T, OffsetItem, Buffer>
{
    fn len(&self) -> usize {
        self.offsets
            .len()
            .checked_sub(1)
            .expect("offset len underflow")
    }
}

#[expect(missing_debug_implementations)]
pub struct OffsetIntoIter<T: Collection, OffsetItem: Offset, Buffer: BufferType> {
    data: T,
    offsets: <Buffer::Buffer<OffsetItem> as Collection>::IntoIter,
    position: OffsetItem,
}

impl<T: Collection, OffsetItem: Offset, Buffer: BufferType> Iterator
    for OffsetIntoIter<T, OffsetItem, Buffer>
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

impl<T: Collection, OffsetItem: Offset, Buffer: BufferType> Collection
    for Offsets<T, OffsetItem, Buffer>
{
    type View<'collection>
        = OffsetView<'collection, T, OffsetItem, Buffer>
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

    fn iter(&self) -> Self::Iter<'_> {
        let len = self.len();
        (0..len)
            .zip(iter::repeat(self))
            .map(|(index, offsets)| offsets.view(index).expect("index in range"))
    }

    type IntoIter = OffsetIntoIter<T, OffsetItem, Buffer>;

    fn into_iter(self) -> Self::IntoIter {
        let Self { data, offsets } = self;
        let mut iter = offsets.into_iter();
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
pub struct OffsetView<'collection, T: Collection, OffsetItem: Offset, Buffer: BufferType> {
    collection: &'collection Offsets<T, OffsetItem, Buffer>,
    start: usize,
    end: usize,
}

impl<T: Collection, OffsetItem: Offset, Buffer: BufferType> Clone
    for OffsetView<'_, T, OffsetItem, Buffer>
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Collection, OffsetItem: Offset, Buffer: BufferType> Copy
    for OffsetView<'_, T, OffsetItem, Buffer>
{
}

impl<T: Collection, OffsetItem: Offset, Buffer: BufferType> IntoOwned<Vec<<T as Collection>::Owned>>
    for OffsetView<'_, T, OffsetItem, Buffer>
{
    fn into_owned(self) -> Vec<<T as Collection>::Owned> {
        Collection::into_iter(self).collect()
    }
}

impl<T: Collection, OffsetItem: Offset, Buffer: BufferType> Length
    for OffsetView<'_, T, OffsetItem, Buffer>
{
    fn len(&self) -> usize {
        self.end - self.start
    }
}

impl<T: Collection, OffsetItem: Offset, Buffer: BufferType> Collection
    for OffsetView<'_, T, OffsetItem, Buffer>
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

    fn iter(&self) -> Self::Iter<'_> {
        (0..self.len())
            .zip(iter::repeat(self))
            .map(|(index, collection)| collection.view(index).expect("index in range"))
    }

    type IntoIter = Map<Zip<Range<usize>, Repeat<Self>>, fn((usize, Self)) -> Self::Owned>;

    fn into_iter(self) -> Self::IntoIter {
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

        let views = offsets.iter().collect::<Vec<_>>();
        assert_eq!(views[0].into_owned(), vec![1, 2]);
        assert_eq!(views[1].into_iter().collect::<Vec<_>>(), vec![3]);
        assert_eq!(views[2].iter().collect::<Vec<_>>(), vec![4]);
    }
}
