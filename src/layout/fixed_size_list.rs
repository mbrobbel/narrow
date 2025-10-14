use crate::{
    buffer::{Buffer, VecBuffer},
    collection::{Collection, CollectionAlloc, CollectionRealloc, flatten::Flatten},
    layout::{Layout, MemoryLayout},
    length::Length,
    nullability::{NonNullable, Nullability},
};

#[derive(Debug)]
pub struct FixedSizeList<
    T: Layout,
    const N: usize,
    Nulls: Nullability = NonNullable,
    Storage: Buffer = VecBuffer,
>(Nulls::Collection<Flatten<T::Memory<Storage>, N>, Storage>);

impl<T: Layout, const N: usize, Nulls: Nullability, Storage: Buffer> MemoryLayout
    for FixedSizeList<T, N, Nulls, Storage>
{
}

impl<T: Layout, const N: usize, Nulls: Nullability, Storage: Buffer> Clone
    for FixedSizeList<T, N, Nulls, Storage>
where
    Nulls::Collection<Flatten<T::Memory<Storage>, N>, Storage>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Layout, const N: usize, Nulls: Nullability, Storage: Buffer> Default
    for FixedSizeList<T, N, Nulls, Storage>
where
    Nulls::Collection<Flatten<T::Memory<Storage>, N>, Storage>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Layout, const N: usize, Nulls: Nullability, Storage: Buffer> Extend<Nulls::Item<[T; N]>>
    for FixedSizeList<T, N, Nulls, Storage>
where
    Nulls::Collection<Flatten<T::Memory<Storage>, N>, Storage>: Extend<Nulls::Item<[T; N]>>,
{
    fn extend<I: IntoIterator<Item = Nulls::Item<[T; N]>>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: Layout, const N: usize, Nulls: Nullability, Storage: Buffer>
    FromIterator<Nulls::Item<[T; N]>> for FixedSizeList<T, N, Nulls, Storage>
where
    Nulls::Collection<Flatten<T::Memory<Storage>, N>, Storage>: FromIterator<Nulls::Item<[T; N]>>,
{
    fn from_iter<I: IntoIterator<Item = Nulls::Item<[T; N]>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: Layout, const N: usize, Nulls: Nullability, Storage: Buffer> Length
    for FixedSizeList<T, N, Nulls, Storage>
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Layout, const N: usize, Nulls: Nullability, Storage: Buffer> Collection
    for FixedSizeList<T, N, Nulls, Storage>
{
    type View<'collection>
        = <Nulls::Collection<Flatten<T::Memory<Storage>, N>, Storage> as Collection>::View<
        'collection,
    >
    where
        Self: 'collection;

    type Owned = Nulls::Item<[T; N]>;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.0.view(index)
    }

    type Iter<'collection>
        = <Nulls::Collection<Flatten<T::Memory<Storage>, N>, Storage> as Collection>::Iter<
        'collection,
    >
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        self.0.iter_views()
    }

    type IntoIter =
        <Nulls::Collection<Flatten<T::Memory<Storage>, N>, Storage> as Collection>::IntoIter;

    fn into_iter_owned(self) -> Self::IntoIter {
        self.0.into_iter_owned()
    }
}

impl<T: Layout, const N: usize, Nulls: Nullability, Storage: Buffer> CollectionAlloc
    for FixedSizeList<T, N, Nulls, Storage>
where
    Nulls::Collection<Flatten<T::Memory<Storage>, N>, Storage>: CollectionAlloc,
{
    fn with_capacity(capacity: usize) -> Self {
        Self(Nulls::Collection::<Flatten<T::Memory<Storage>, N>, Storage>::with_capacity(capacity))
    }
}

impl<T: Layout, const N: usize, Nulls: Nullability, Storage: Buffer> CollectionRealloc
    for FixedSizeList<T, N, Nulls, Storage>
where
    Nulls::Collection<Flatten<T::Memory<Storage>, N>, Storage>: CollectionRealloc,
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
        round_trip::<FixedSizeList<_, _>, _>([[1], [2], [3], [4]]);
        round_trip::<FixedSizeList<_, _, Nullable>, _>([Some([1, 2, 3, 4]), None]);
        round_trip::<FixedSizeList<_, _>, _>([[Some(1)], [None], [Some(3)], [Some(4)]]);
        round_trip::<FixedSizeList<_, _, Nullable>, _>([
            Some([Some(1)]),
            None,
            Some([None]),
            Some([Some(4)]),
        ]);
        round_trip::<FixedSizeList<_, _, Nullable>, _>([
            Some([vec![1, 2], vec![3, 4]]),
            None,
            Some([vec![5, 6, 7], vec![8]]),
        ]);
    }
}
