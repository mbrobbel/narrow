//! Sequences of values with known length all having the same type.

use core::fmt::Debug;

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

impl<T: Layout, Storage: Buffer> Debug for Array<T, Storage>
where
    T::Memory<Storage>: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Array").field(&self.0).finish()
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

impl<T: Layout, U, Storage: Buffer> Extend<U> for Array<T, Storage>
where
    T::Memory<Storage>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: Layout, Storage: Buffer> Length for Array<T, Storage> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Layout, U, Storage: Buffer> FromIterator<U> for Array<T, Storage>
where
    T::Memory<Storage>: FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
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
    extern crate alloc;

    use alloc::vec;

    use crate::{collection::tests::round_trip, fixed_size::FixedSizeArray};

    use super::*;

    #[test]
    fn collection() {
        // Fixed size primitive
        round_trip::<Array<_>, _>([1, 2, 3, 4]);
        round_trip::<Array<_>, _>([Some(1), None, Some(3), Some(4)]);
        round_trip::<Array<FixedSizeArray<u8, 4>>, _>([[1, 2, 3, 4].into(), [5, 6, 7, 8].into()]);
        round_trip::<Array<Option<FixedSizeArray<u8, 4>>>, _>([Some([1, 2, 3, 4].into()), None]);

        // Fixed size list
        round_trip::<Array<_>, _>([[1, 2, 3, 4], [5, 6, 7, 8]]);
        round_trip::<Array<_>, _>([Some([1, 2, 3, 4]), None]);
        round_trip::<Array<_>, _>([Some([Some(1), None, Some(3), Some(4)]), None]);
        round_trip::<Array<_>, _>([
            Some([Some(vec![1, 2]), None, Some(vec![3]), Some(vec![4])]),
            None,
            Some([None, Some(vec![5, 6, 7, 8]), None, None]),
        ]);

        // Variable size binary
        round_trip::<Array<_>, _>([vec![1_u8, 2, 3, 4], vec![5, 6, 7, 8]]);
        round_trip::<Array<_>, _>([Some(vec![1_u8, 2, 3, 4]), None]);

        // Variable size list
        round_trip::<Array<_>, _>([vec![1, 2, 3, 4], vec![5, 6, 7, 8]]);
        round_trip::<Array<_>, _>([Some(vec![1, 2, 3, 4]), None]);
        round_trip::<Array<_>, _>([vec![vec![1, 2], vec![3, 4]], vec![vec![5, 6, 7], vec![8]]]);
        round_trip::<Array<_>, _>([Some(vec![Some(1), None, Some(3), Some(4)]), None]);
        round_trip::<Array<_>, _>([
            Some(vec![
                Some(vec![1, 2]),
                None,
                Some(vec![3, 4, 5, 6]),
                Some(vec![7, 8]),
            ]),
            None,
            Some(vec![None]),
            Some(vec![Some(vec![])]),
            None,
        ]);
        round_trip::<Array<_>, _>([vec![[1, 2], [3, 4]], vec![[5, 6], [7, 8]]]);
        round_trip::<Array<_>, _>([vec![Some([1, 2]), None], vec![None, Some([7, 8])]]);
        round_trip::<Array<_>, _>([Some(vec![Some([1, 2]), None]), None]);
    }
}
