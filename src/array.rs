//! Sequences of values with known length all having the same type.

use std::fmt::Debug;

use crate::{
    buffer::{Buffer, VecBuffer},
    collection::{Collection, CollectionAlloc, CollectionRealloc},
    layout::Layout,
    length::Length,
};

/// An array of items `T`, stored using their [`Layout`] memory.
pub struct Array<T: Layout, Storage: Buffer = VecBuffer>(T::Memory<Storage>);

impl<T: Layout, Storage: Buffer> Clone for Array<T, Storage>
where
    T::Memory<Storage>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Layout, Storage: Buffer> Debug for Array<T, Storage> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Array").finish_non_exhaustive()
    }
}

impl<T: Layout, Storage: Buffer> Default for Array<T, Storage>
where
    T::Memory<Storage>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Layout, Storage: Buffer> Extend<T> for Array<T, Storage>
where
    T::Memory<Storage>: Extend<T>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: Layout, Storage: Buffer> Length for Array<T, Storage> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Layout, Storage: Buffer> FromIterator<T> for Array<T, Storage>
where
    T::Memory<Storage>: FromIterator<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: Layout, Storage: Buffer> Collection for Array<T, Storage> {
    type View<'collection>
        = <T::Memory<Storage> as Collection>::View<'collection>
    where
        Self: 'collection;

    type Owned = T;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.0.view(index)
    }

    type Iter<'collection>
        = <T::Memory<Storage> as Collection>::Iter<'collection>
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        self.0.iter_views()
    }

    type IntoIter = <T::Memory<Storage> as Collection>::IntoIter;

    fn into_iter_owned(self) -> Self::IntoIter {
        self.0.into_iter_owned()
    }
}

impl<T: Layout, Storage: Buffer> CollectionAlloc for Array<T, Storage>
where
    T::Memory<Storage>: CollectionAlloc,
{
    fn with_capacity(capacity: usize) -> Self {
        Self(T::Memory::<Storage>::with_capacity(capacity))
    }
}

impl<T: Layout, Storage: Buffer> CollectionRealloc for Array<T, Storage>
where
    T::Memory<Storage>: CollectionRealloc,
{
    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
}

#[cfg(test)]
mod tests {
    use crate::collection::tests::round_trip;

    use super::*;

    #[test]
    fn collection() {
        // Fixed size primitive
        round_trip::<Array<i32>, _>([1, 2, 3, 4]);
        round_trip::<Array<Option<i32>>, _>([Some(1), None, Some(3), Some(4)]);
        round_trip::<Array<[i32; 4]>, _>([[1, 2, 3, 4], [5, 6, 7, 8]]);
        round_trip::<Array<Option<[i32; 4]>>, _>([Some([1, 2, 3, 4]), None]);

        // Variable size binary
        round_trip::<Array<Vec<u8>>, _>([vec![1, 2, 3, 4], vec![5, 6, 7, 8]]);
        round_trip::<Array<Option<Vec<u8>>>, _>([Some(vec![1, 2, 3, 4]), None]);
    }
}
