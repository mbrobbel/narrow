extern crate alloc;

use alloc::vec::Vec;

use crate::{
    buffer::{Buffer, VecBuffer},
    collection::{Collection, CollectionAlloc, CollectionRealloc},
    layout::{Layout, MemoryLayout},
    length::Length,
    nullability::{NonNullable, Nullability},
    offset::{Offset, Offsets},
};

#[derive(Debug)]
pub struct VariableSizeList<
    T: Layout,
    Nulls: Nullability = NonNullable,
    OffsetItem: Offset = i32,
    Storage: Buffer = VecBuffer,
>(Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>);

impl<T: Layout, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> MemoryLayout
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
{
}

impl<T: Layout, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> Default
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Layout, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> Extend<Nulls::Item<Vec<T>>>
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>:
        Extend<Nulls::Item<Vec<T>>>,
{
    fn extend<I: IntoIterator<Item = Nulls::Item<Vec<T>>>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: Layout, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer>
    FromIterator<Nulls::Item<Vec<T>>> for VariableSizeList<T, Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>:
        FromIterator<Nulls::Item<Vec<T>>>,
{
    fn from_iter<I: IntoIterator<Item = Nulls::Item<Vec<T>>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: Layout, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> Length
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Layout, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> Collection
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

impl<T: Layout, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> CollectionAlloc
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>: CollectionAlloc,
{
    fn with_capacity(capacity: usize) -> Self {
        Self(Nulls::Collection::<
            Offsets<T::Memory<Storage>, OffsetItem, Storage>,
            Storage,
        >::with_capacity(capacity))
    }
}

impl<T: Layout, Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> CollectionRealloc
    for VariableSizeList<T, Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<T::Memory<Storage>, OffsetItem, Storage>, Storage>: CollectionRealloc,
{
    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec;

    use crate::{collection::tests::round_trip, nullability::Nullable};

    use super::*;

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
