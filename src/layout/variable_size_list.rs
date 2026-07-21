extern crate alloc;

use alloc::vec::Vec;
use core::fmt::Debug;

use crate::{
    buffer::{Buffer, BufferRef, VecBuffer},
    collection::{AllocError, Collection, CollectionAllocIn, CollectionRealloc},
    layout::{ArrayItem, MemoryLayout},
    length::Length,
    nullability::{NonNullable, Nullability},
    offset::{Offset, Offsets},
};

pub struct VariableSizeList<
    T: ArrayItem,
    Nulls: Nullability = NonNullable,
    OffsetItem: Offset = i32,
    Storage: Buffer = VecBuffer,
>(Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>);

impl<T: ArrayItem, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> MemoryLayout
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
{
}

impl<T: ArrayItem, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer>
    VariableSizeList<T, Nulls, OffsetItem, Storage>
{
    /// Constructs a [`VariableSizeList`] from its backing collection.
    #[must_use]
    pub fn from_buffer(
        buffer: Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>,
    ) -> Self {
        Self(buffer)
    }

    /// Returns the backing collection of this [`VariableSizeList`].
    ///
    /// This is the inverse of [`VariableSizeList::from_buffer`].
    #[must_use]
    pub fn into_buffer(
        self,
    ) -> Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage> {
        self.0
    }
}

impl<T: ArrayItem, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> BufferRef
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
{
    type Buffer = Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>;

    fn buffer_ref(&self) -> &Self::Buffer {
        &self.0
    }
}

impl<T: ArrayItem, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> Debug
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VariableSizeList").field(&self.0).finish()
    }
}

impl<T: ArrayItem, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> Default
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: ArrayItem, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer>
    Extend<Nulls::Item<Vec<T>>> for VariableSizeList<T, Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>:
        Extend<Nulls::Item<Vec<T>>>,
{
    fn extend<I: IntoIterator<Item = Nulls::Item<Vec<T>>>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: ArrayItem, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer>
    FromIterator<Nulls::Item<Vec<T>>> for VariableSizeList<T, Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>:
        FromIterator<Nulls::Item<Vec<T>>>,
{
    fn from_iter<I: IntoIterator<Item = Nulls::Item<Vec<T>>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: ArrayItem, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> Length
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: ArrayItem, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> Collection
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
{
    type View<'collection> = <Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage> as Collection>::View<'collection>
    where
        Self: 'collection;

    type Owned = <Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage> as Collection>::Owned;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.0.view(index)
    }

    type Iter<'collection> = <Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage> as Collection>::Iter<'collection>
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        self.0.iter_views()
    }

    type IntoIter = <Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage> as Collection>::IntoIter;

    fn into_iter_owned(self) -> Self::IntoIter {
        self.0.into_iter_owned()
    }
}

impl<T: ArrayItem, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> CollectionAllocIn
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>: CollectionAllocIn,
{
    type Alloc = <Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage> as CollectionAllocIn>::Alloc;

    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self {
        Self(Nulls::Collection::<
            Offsets<T::Memory<Storage>, OffsetItem, Storage>,
            Storage,
        >::with_capacity_in(capacity, alloc))
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, alloc: Self::Alloc) -> Self {
        Self(Nulls::Collection::<
            Offsets<T::Memory<Storage>, OffsetItem, Storage>,
            Storage,
        >::from_iter_in(iter, alloc))
    }

    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, AllocError> {
        Nulls::Collection::<
            Offsets<T::Memory<Storage>, OffsetItem, Storage>,
            Storage,
        >::try_with_capacity_in(capacity, alloc)
        .map(Self)
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, AllocError> {
        Nulls::Collection::<
            Offsets<T::Memory<Storage>, OffsetItem, Storage>,
            Storage,
        >::try_from_iter_in(iter, alloc)
        .map(Self)
    }
}

impl<T: ArrayItem, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> CollectionRealloc
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>: CollectionRealloc,
{
    fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError> {
        self.0.try_reserve(additional)
    }

    fn try_extend<I: IntoIterator<Item = Self::Owned>>(
        &mut self,
        iter: I,
    ) -> Result<(), AllocError> {
        self.0.try_extend(iter)
    }

    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec;

    use crate::{collection::tests::round_trip, nullability::Nullable};

    use super::*;

    #[test]
    fn from_buffer() {
        let list = [vec![1, 2], vec![3, 4, 5]]
            .into_iter()
            .collect::<VariableSizeList<i32>>();
        let restored = VariableSizeList::<i32>::from_buffer(list.into_buffer());
        assert_eq!(restored.len(), 2);
        assert_eq!(restored.owned(0), Some(vec![1, 2]));
        assert_eq!(restored.owned(1), Some(vec![3, 4, 5]));
    }

    #[test]
    fn collection() {
        round_trip::<VariableSizeList<u16>, _>([vec![1, 2, 3, 4], vec![5, 6, 7, 8]]);
        round_trip::<VariableSizeList<u16, Nullable>, _>([Some(vec![1, 2, 3, 4]), None]);
        round_trip::<VariableSizeList<Vec<u16>>, _>([
            vec![vec![1, 2], vec![3, 4]],
            vec![vec![5, 6, 7], vec![8]],
        ]);
        round_trip::<VariableSizeList<Vec<Vec<u16>>>, _>([
            vec![vec![vec![1, 2], vec![3, 4]], vec![]],
            vec![vec![vec![5, 6, 7], vec![8]], vec![]],
        ]);
    }
}
