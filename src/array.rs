//! Sequences of values with known length all having the same type.

use core::fmt::Debug;

use crate::{
    buffer::{Buffer, VecBuffer},
    collection::{AllocError, Collection, CollectionAllocIn, CollectionRealloc},
    layout::Layout,
    length::Length,
};

/// An array of items `T`, stored using their [`Layout`] memory.
pub struct Array<T: Layout, Storage: Buffer = VecBuffer>(T::Memory<Storage>);

impl<T: Layout, Storage: Buffer> Array<T, Storage> {
    /// Constructs an [`Array`] from its backing memory layout.
    #[must_use]
    pub fn from_buffer(memory: T::Memory<Storage>) -> Self {
        Self(memory)
    }

    /// Returns the backing memory layout of this [`Array`].
    ///
    /// This is the inverse of [`Array::from_buffer`].
    #[must_use]
    pub fn into_buffer(self) -> T::Memory<Storage> {
        self.0
    }
}

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

impl<T: Layout, Storage: Buffer> CollectionAllocIn for Array<T, Storage>
where
    T::Memory<Storage>: CollectionAllocIn,
{
    type Alloc = <T::Memory<Storage> as CollectionAllocIn>::Alloc;

    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self {
        Self(T::Memory::<Storage>::with_capacity_in(capacity, alloc))
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, alloc: Self::Alloc) -> Self {
        Self(T::Memory::<Storage>::from_iter_in(iter, alloc))
    }

    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, AllocError> {
        T::Memory::<Storage>::try_with_capacity_in(capacity, alloc).map(Self)
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, AllocError> {
        T::Memory::<Storage>::try_from_iter_in(iter, alloc).map(Self)
    }
}

impl<T: Layout, Storage: Buffer> CollectionRealloc for Array<T, Storage>
where
    T::Memory<Storage>: CollectionRealloc,
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

    use crate::{collection::tests::round_trip, fixed_size::FixedSizeArray};

    use super::*;

    #[test]
    fn from_buffer() {
        let array = [1, 2, 3, 4].into_iter().collect::<Array<i32>>();
        let restored = Array::<i32>::from_buffer(array.into_buffer());
        assert_eq!(restored.len(), 4);
        assert_eq!(restored.owned(0), Some(1));
        assert_eq!(restored.owned(3), Some(4));
    }

    #[test]
    fn alloc_in_nested() {
        type Nested = Array<Option<alloc::vec::Vec<i32>>>;

        let infallible =
            <Nested as CollectionAllocIn>::from_iter_in([Some(alloc::vec![1, 2]), None], ());
        assert_eq!(infallible.owned(0), Some(Some(alloc::vec![1, 2])));
        assert_eq!(infallible.owned(1), Some(None));

        let mut array =
            <Nested as CollectionAllocIn>::try_from_iter_in([Some(alloc::vec![1, 2]), None], ())
                .expect("allocation succeeds");
        array
            .try_extend([Some(alloc::vec![3, 4, 5])])
            .expect("allocation succeeds");
        assert_eq!(array.owned(2), Some(Some(alloc::vec![3, 4, 5])));
    }

    #[test]
    fn collection() {
        // Boolean
        round_trip::<Array<_>, _>([true, false, true, true]);
        round_trip::<Array<_>, _>([Some(true), None, Some(false), Some(true)]);

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
