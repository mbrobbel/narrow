use core::{borrow::Borrow, fmt::Debug};

use crate::{
    buffer::{Buffer, VecBuffer},
    collection::{Collection, CollectionAlloc, CollectionRealloc},
    fixed_size::FixedSize,
    layout::MemoryLayout,
    length::Length,
    nullability::{NonNullable, Nullability},
};

/// A collection of `FixedSize` items.
///
/// <https://arrow.apache.org/docs/format/Columnar.html#fixed-size-primitive-layout>
pub struct FixedSizePrimitive<
    T: FixedSize,
    Nulls: Nullability = NonNullable,
    Storage: Buffer = VecBuffer,
>(Nulls::Collection<Storage::For<T>, Storage>);

impl<T: FixedSize, Nulls: Nullability, Storage: Buffer> MemoryLayout
    for FixedSizePrimitive<T, Nulls, Storage>
{
}

impl<T: FixedSize, Storage: Buffer> Borrow<[T]> for FixedSizePrimitive<T, NonNullable, Storage> {
    fn borrow(&self) -> &[T] {
        self.0.borrow()
    }
}

impl<T: FixedSize, Nulls: Nullability, Storage: Buffer> Debug
    for FixedSizePrimitive<T, Nulls, Storage>
where
    Nulls::Collection<Storage::For<T>, Storage>: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("FixedSizePrimitive").field(&self.0).finish()
    }
}

impl<T: FixedSize, Nulls: Nullability, Storage: Buffer> Default
    for FixedSizePrimitive<T, Nulls, Storage>
where
    Nulls::Collection<Storage::For<T>, Storage>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: FixedSize, Nulls: Nullability, Storage: Buffer> Length
    for FixedSizePrimitive<T, Nulls, Storage>
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: FixedSize, Nulls: Nullability, Storage: Buffer> FromIterator<Nulls::Item<T>>
    for FixedSizePrimitive<T, Nulls, Storage>
where
    Nulls::Collection<Storage::For<T>, Storage>: FromIterator<Nulls::Item<T>>,
{
    fn from_iter<I: IntoIterator<Item = Nulls::Item<T>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: FixedSize, Nulls: Nullability, Storage: Buffer> Extend<Nulls::Item<T>>
    for FixedSizePrimitive<T, Nulls, Storage>
where
    Nulls::Collection<Storage::For<T>, Storage>: Extend<Nulls::Item<T>>,
{
    fn extend<I: IntoIterator<Item = Nulls::Item<T>>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: FixedSize, Nulls: Nullability, Storage: Buffer> Collection
    for FixedSizePrimitive<T, Nulls, Storage>
{
    type View<'collection>
        = <Nulls::Collection<Storage::For<T>, Storage> as Collection>::View<'collection>
    where
        Self: 'collection;

    type Owned = <Nulls::Collection<Storage::For<T>, Storage> as Collection>::Owned;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.0.view(index)
    }

    type Iter<'collection>
        = <Nulls::Collection<Storage::For<T>, Storage> as Collection>::Iter<'collection>
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        self.0.iter_views()
    }

    type IntoIter = <Nulls::Collection<Storage::For<T>, Storage> as Collection>::IntoIter;

    fn into_iter_owned(self) -> Self::IntoIter {
        self.0.into_iter_owned()
    }
}

impl<T: FixedSize, Nulls: Nullability, Storage: Buffer> CollectionAlloc
    for FixedSizePrimitive<T, Nulls, Storage>
where
    Nulls::Collection<Storage::For<T>, Storage>: CollectionAlloc,
{
    fn with_capacity(capacity: usize) -> Self {
        Self(Nulls::Collection::with_capacity(capacity))
    }
}

impl<T: FixedSize, Nulls: Nullability, Storage: Buffer> CollectionRealloc
    for FixedSizePrimitive<T, Nulls, Storage>
where
    Nulls::Collection<Storage::For<T>, Storage>: CollectionRealloc,
{
    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
}

#[cfg(test)]
mod tests {
    use crate::{collection::tests::round_trip, fixed_size::FixedSizeArray, nullability::Nullable};

    use super::*;

    #[test]
    fn collection() {
        round_trip::<FixedSizePrimitive<_>, _>([1, 2, 3, 4]);
        round_trip::<FixedSizePrimitive<_, Nullable>, _>([Some(1), None, Some(3), Some(4)]);
        round_trip::<FixedSizePrimitive<FixedSizeArray<_, _>>, _>([
            [1, 2, 3, 4].into(),
            [5, 6, 7, 8].into(),
        ]);
        round_trip::<FixedSizePrimitive<FixedSizeArray<_, _>, Nullable>, _>([
            Some([1, 2, 3, 4].into()),
            None,
        ]);
    }
}
