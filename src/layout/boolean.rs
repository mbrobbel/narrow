use core::fmt::Debug;

use crate::{
    bitmap::Bitmap,
    buffer::{Buffer, VecBuffer},
    collection::{Collection, CollectionAlloc, CollectionRealloc},
    layout::MemoryLayout,
    length::Length,
    nullability::{NonNullable, Nullability},
};

/// A collection of booleans, stored as bits in a [`Bitmap`].
///
/// <https://arrow.apache.org/docs/format/Columnar.html#fixed-size-primitive-layout>
pub struct Boolean<Nulls: Nullability = NonNullable, Storage: Buffer = VecBuffer>(
    Nulls::Collection<Bitmap<Storage>, Storage>,
);

impl<Nulls: Nullability, Storage: Buffer> MemoryLayout for Boolean<Nulls, Storage> {}

impl<Nulls: Nullability, Storage: Buffer> Debug for Boolean<Nulls, Storage>
where
    Nulls::Collection<Bitmap<Storage>, Storage>: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Boolean").field(&self.0).finish()
    }
}

impl<Nulls: Nullability, Storage: Buffer> Default for Boolean<Nulls, Storage>
where
    Nulls::Collection<Bitmap<Storage>, Storage>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Nulls: Nullability, Storage: Buffer> Length for Boolean<Nulls, Storage> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<Nulls: Nullability, Storage: Buffer> FromIterator<Nulls::Item<bool>>
    for Boolean<Nulls, Storage>
where
    Nulls::Collection<Bitmap<Storage>, Storage>: FromIterator<Nulls::Item<bool>>,
{
    fn from_iter<I: IntoIterator<Item = Nulls::Item<bool>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<Nulls: Nullability, Storage: Buffer> Extend<Nulls::Item<bool>> for Boolean<Nulls, Storage>
where
    Nulls::Collection<Bitmap<Storage>, Storage>: Extend<Nulls::Item<bool>>,
{
    fn extend<I: IntoIterator<Item = Nulls::Item<bool>>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<Nulls: Nullability, Storage: Buffer> Collection for Boolean<Nulls, Storage> {
    type View<'collection>
        = <Nulls::Collection<Bitmap<Storage>, Storage> as Collection>::View<'collection>
    where
        Self: 'collection;

    type Owned = <Nulls::Collection<Bitmap<Storage>, Storage> as Collection>::Owned;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.0.view(index)
    }

    type Iter<'collection>
        = <Nulls::Collection<Bitmap<Storage>, Storage> as Collection>::Iter<'collection>
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        self.0.iter_views()
    }

    type IntoIter = <Nulls::Collection<Bitmap<Storage>, Storage> as Collection>::IntoIter;

    fn into_iter_owned(self) -> Self::IntoIter {
        self.0.into_iter_owned()
    }
}

impl<Nulls: Nullability, Storage: Buffer> CollectionAlloc for Boolean<Nulls, Storage>
where
    Nulls::Collection<Bitmap<Storage>, Storage>: CollectionAlloc,
{
    fn with_capacity(capacity: usize) -> Self {
        Self(Nulls::Collection::with_capacity(capacity))
    }
}

impl<Nulls: Nullability, Storage: Buffer> CollectionRealloc for Boolean<Nulls, Storage>
where
    Nulls::Collection<Bitmap<Storage>, Storage>: CollectionRealloc,
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
        round_trip::<Boolean, _>([true, false, true, true]);
        round_trip::<Boolean<Nullable>, _>([Some(true), None, Some(false), Some(true)]);
    }
}
