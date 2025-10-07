use crate::{
    buffer::{Buffer, VecBuffer},
    collection::{Collection, CollectionAlloc, CollectionRealloc},
    layout::MemoryLayout,
    length::Length,
    nullability::{NonNullable, Nullability},
    offset::{Offset, Offsets},
};

#[derive(Debug)]
pub struct VariableSizeBinary<
    Nulls: Nullability = NonNullable,
    OffsetItem: Offset = i32,
    Storage: Buffer = VecBuffer,
>(Nulls::Collection<Offsets<Storage::For<u8>, OffsetItem, Storage>, Storage>);

impl<Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> MemoryLayout
    for VariableSizeBinary<Nulls, OffsetItem, Storage>
{
}

impl<Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> Default
    for VariableSizeBinary<Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<Storage::For<u8>, OffsetItem, Storage>, Storage>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> Extend<Nulls::Item<Vec<u8>>>
    for VariableSizeBinary<Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<Storage::For<u8>, OffsetItem, Storage>, Storage>:
        Extend<Nulls::Item<Vec<u8>>>,
{
    fn extend<I: IntoIterator<Item = Nulls::Item<Vec<u8>>>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> FromIterator<Nulls::Item<Vec<u8>>>
    for VariableSizeBinary<Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<Storage::For<u8>, OffsetItem, Storage>, Storage>:
        FromIterator<Nulls::Item<Vec<u8>>>,
{
    fn from_iter<I: IntoIterator<Item = Nulls::Item<Vec<u8>>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> Length
    for VariableSizeBinary<Nulls, OffsetItem, Storage>
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> Collection
    for VariableSizeBinary<Nulls, OffsetItem, Storage>
{
    type View<'collection> = <Nulls::Collection<Offsets<Storage::For<u8>, OffsetItem, Storage>, Storage> as Collection>::View<'collection>
    where
        Self: 'collection;

    type Owned = <Nulls::Collection<Offsets<Storage::For<u8>, OffsetItem, Storage>, Storage> as Collection>::Owned;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.0.view(index)
    }

    type Iter<'collection> = <Nulls::Collection<Offsets<Storage::For<u8>, OffsetItem, Storage>, Storage> as Collection>::Iter<'collection>
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        self.0.iter_views()
    }

    type IntoIter = <Nulls::Collection<Offsets<Storage::For<u8>, OffsetItem, Storage>, Storage> as Collection>::IntoIter;

    fn into_iter_owned(self) -> Self::IntoIter {
        self.0.into_iter_owned()
    }
}

impl<Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> CollectionAlloc
    for VariableSizeBinary<Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<Storage::For<u8>, OffsetItem, Storage>, Storage>: CollectionAlloc,
{
    fn with_capacity(capacity: usize) -> Self {
        Self(Nulls::Collection::<
            Offsets<Storage::For<u8>, OffsetItem, Storage>,
            Storage,
        >::with_capacity(capacity))
    }
}

impl<Nulls: Nullability, OffsetItem: Offset, Storage: Buffer> CollectionRealloc
    for VariableSizeBinary<Nulls, OffsetItem, Storage>
where
    Nulls::Collection<Offsets<Storage::For<u8>, OffsetItem, Storage>, Storage>: CollectionRealloc,
{
    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
}

#[cfg(test)]
mod tests {
    use crate::{collection::tests::round_trip, nullability::Nullable};

    use super::*;

    #[test]
    fn collection() {
        round_trip::<VariableSizeBinary, _>([vec![1, 2, 3, 4], vec![5, 6, 7, 8]]);
        round_trip::<VariableSizeBinary<Nullable>, _>([Some(vec![1, 2, 3, 4]), None]);
    }
}
