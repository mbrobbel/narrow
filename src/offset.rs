use std::{
    fmt::{self, Debug},
    iter::{self, Map, Repeat, Zip},
    num::TryFromIntError,
    ops::Range,
};

use crate::{
    buffer::{BufferType, VecBuffer},
    collection::{Collection, IntoOwned},
    fixed_size::FixedSize,
    length::Length,
    nullability::{NonNullable, Nullability},
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

pub struct Offsets<
    T: Collection,
    Nulls: Nullability = NonNullable,
    OffsetItem: Offset = i32,
    Buffer: BufferType = VecBuffer,
> {
    data: T,
    offsets: Nulls::Collection<Buffer::Buffer<OffsetItem>, Buffer>,
}

impl<T: Collection, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType> Debug
    for Offsets<T, Nulls, OffsetItem, Buffer>
where
    T: Debug,
    Nulls::Collection<Buffer::Buffer<OffsetItem>, Buffer>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Offset")
            .field("data", &self.data)
            .field("offsets", &self.offsets)
            .finish()
    }
}

impl<T: Collection, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType> Default
    for Offsets<T, Nulls, OffsetItem, Buffer>
where
    T: Default,
    Nulls::Item<OffsetItem>: Default,
    Nulls::Collection<Buffer::Buffer<OffsetItem>, Buffer>:
        Default + Extend<Nulls::Item<OffsetItem>>,
{
    fn default() -> Self {
        let mut offsets: Nulls::Collection<Buffer::Buffer<OffsetItem>, Buffer> = Default::default();
        offsets.extend(iter::once(Nulls::Item::default()));
        Self {
            data: T::default(),
            offsets,
        }
    }
}

impl<T: Collection, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType> Length
    for Offsets<T, Nulls, OffsetItem, Buffer>
{
    fn len(&self) -> usize {
        self.offsets
            .len()
            .checked_sub(1)
            .expect("offset len underflow")
    }
}
#[expect(missing_debug_implementations)]
pub struct OffsetIntoIter<T: Collection, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType>
{
    data: T,
    offsets: <Nulls::Collection<Buffer::Buffer<OffsetItem>, Buffer> as Collection>::IntoIter,
    position: Nulls::Item<OffsetItem>,
}

impl<T: Collection, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType> Iterator
    for OffsetIntoIter<T, Nulls, OffsetItem, Buffer>
where
    Nulls::Item<OffsetItem>: Copy,
{
    type Item = Nulls::Item<Vec<T::Owned>>;

    fn next(&mut self) -> Option<Self::Item> {
        let end = self.offsets.next()?;
        let item = Nulls::zip_with(self.position, end, |(start, end)| {
            let (start, end) = (start.as_usize(), end.as_usize());
            (start..end)
                .map(|index| self.data.view(index).expect("out of bounds").into_owned())
                .collect::<Vec<T::Owned>>()
        });
        self.position = end;
        Some(item)
    }
}

impl<T: Collection, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType> Collection
    for Offsets<T, Nulls, OffsetItem, Buffer>
where
    Nulls::Item<OffsetItem>: Copy,
    for<'collection> Nulls::Item<OffsetView<'collection, T, Nulls, OffsetItem, Buffer>>:
        IntoOwned<Nulls::Item<Vec<<T as Collection>::Owned>>>,
{
    type View<'collection>
        = Nulls::Item<OffsetView<'collection, T, Nulls, OffsetItem, Buffer>>
    where
        Self: 'collection;

    type Owned = Nulls::Item<Vec<T::Owned>>;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        let start = self.offsets.owned(index);
        let end = self.offsets.owned(index + 1);
        start.zip(end).map(|(start, end)| {
            Nulls::zip_with(start, end, |(start, end)| OffsetView {
                collection: self,
                start: start.as_usize(),
                end: end.as_usize(),
            })
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
        (0..self.len())
            .zip(iter::repeat(self))
            .map(|(index, offsets)| offsets.view(index).expect("index in range"))
    }

    type IntoIter = OffsetIntoIter<T, Nulls, OffsetItem, Buffer>;

    fn into_iter(self) -> Self::IntoIter {
        let Self { data, offsets } = self;
        let mut offsets = offsets.into_iter();
        let position = offsets
            .next()
            .expect("offset buffer must have at least one value");
        OffsetIntoIter {
            data,
            offsets,
            position,
        }
    }
}

#[expect(missing_debug_implementations)]
pub struct OffsetView<
    'collection,
    T: Collection,
    Nulls: Nullability,
    OffsetItem: Offset,
    Buffer: BufferType,
> {
    collection: &'collection Offsets<T, Nulls, OffsetItem, Buffer>,
    start: usize,
    end: usize,
}

impl<'collection, T: Collection, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType> Clone
    for OffsetView<'collection, T, Nulls, OffsetItem, Buffer>
{
    fn clone(&self) -> Self {
        Self {
            collection: &self.collection,
            start: self.start,
            end: self.end,
        }
    }
}
impl<'collection, T: Collection, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType> Copy
    for OffsetView<'collection, T, Nulls, OffsetItem, Buffer>
{
}

impl<'collection, T: Collection, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType>
    IntoOwned<Vec<<T as Collection>::Owned>>
    for OffsetView<'collection, T, Nulls, OffsetItem, Buffer>
{
    fn into_owned(self) -> Vec<<T as Collection>::Owned> {
        Collection::iter(&self).map(IntoOwned::into_owned).collect()
    }
}

impl<'collection, T: Collection, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType> Length
    for OffsetView<'collection, T, Nulls, OffsetItem, Buffer>
{
    fn len(&self) -> usize {
        self.end - self.start
    }
}

impl<'offsets, T: Collection, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType> Collection
    for OffsetView<'offsets, T, Nulls, OffsetItem, Buffer>
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
    use crate::nullability::Nullable;

    use super::*;

    #[test]
    fn default_len() {
        let non_nullable: Offsets<Vec<u8>, NonNullable> = Offsets::default();
        assert_eq!(non_nullable.len(), 0);
        assert_eq!(non_nullable.data.len(), 0);
        assert_eq!(non_nullable.offsets.len(), 1);

        let non_nullable: Offsets<Vec<u8>, Nullable> = Offsets::default();
        assert_eq!(non_nullable.len(), 0);
        assert_eq!(non_nullable.data.len(), 0);
        assert_eq!(non_nullable.offsets.len(), 1);
    }

    #[test]
    fn view() {
        let non_nullable: Offsets<Vec<i32>, NonNullable> = Offsets {
            data: vec![1, 2, 3, 4, 5],
            offsets: vec![0, 2, 3, 4, 5],
        };

        assert_eq!(non_nullable.len(), 4);

        let slice = non_nullable.view(0).unwrap();
        assert_eq!(slice.start, 0);
        assert_eq!(slice.end, 2);
        assert_eq!(slice.view(0), Some(1));
        assert_eq!(slice.view(1), Some(2));
        assert_eq!(slice.view(2), None);

        let views = non_nullable.iter().collect::<Vec<_>>();
        assert_eq!(views[0].into_owned(), vec![1, 2]);
        assert_eq!(views[1].into_iter().collect::<Vec<_>>(), vec![3]);
        assert_eq!(views[2].iter().collect::<Vec<_>>(), vec![4]);
    }
}
