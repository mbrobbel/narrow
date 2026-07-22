use core::fmt::Debug;

use crate::{
    bitmap::Bitmap,
    buffer::{Buffer, BufferRef, VecBuffer},
    collection::{AllocError, Collection, CollectionAllocIn, CollectionRealloc},
    layout::MemoryLayout,
    length::Length,
    nullability::{NonNullable, Nullability},
};

/// A collection of booleans, stored as bits in a [`Bitmap`].
///
/// <https://arrow.apache.org/docs/format/Columnar.html#fixed-size-primitive-layout>
///
/// # Examples
///
/// ```
/// use narrow::{collection::Collection, layout::boolean::Boolean, nullability::Nullable};
///
/// let values = [true, false].into_iter().collect::<Boolean>();
/// assert_eq!(values.owned(1), Some(false));
/// let values = [Some(true), None].into_iter().collect::<Boolean<Nullable>>();
/// assert_eq!(values.owned(1), Some(None));
/// ```
pub struct Boolean<Nulls: Nullability = NonNullable, Storage: Buffer = VecBuffer>(
    Nulls::Collection<Bitmap<Storage>, Storage>,
);

impl<Nulls: Nullability, Storage: Buffer> MemoryLayout for Boolean<Nulls, Storage> {}

impl<Nulls: Nullability, Storage: Buffer> Boolean<Nulls, Storage> {
    /// Constructs a [`Boolean`] from its backing collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{bitmap::Bitmap, collection::Collection, layout::boolean::Boolean};
    ///
    /// let bitmap = [true, false].into_iter().collect::<Bitmap>();
    /// let values = Boolean::<narrow::nullability::NonNullable>::from_buffer(bitmap);
    /// assert_eq!(values.owned(0), Some(true));
    /// ```
    #[must_use]
    pub fn from_buffer(buffer: Nulls::Collection<Bitmap<Storage>, Storage>) -> Self {
        Self(buffer)
    }

    /// Returns the backing collection of this [`Boolean`].
    ///
    /// This is the inverse of [`Boolean::from_buffer`].
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{layout::boolean::Boolean, length::Length};
    ///
    /// let bitmap = [true, false].into_iter().collect::<Boolean>().into_buffer();
    /// assert_eq!(bitmap.len(), 2);
    /// ```
    #[must_use]
    pub fn into_buffer(self) -> Nulls::Collection<Bitmap<Storage>, Storage> {
        self.0
    }
}

impl<Nulls: Nullability, Storage: Buffer> BufferRef for Boolean<Nulls, Storage> {
    type Buffer = Nulls::Collection<Bitmap<Storage>, Storage>;

    fn buffer_ref(&self) -> &Self::Buffer {
        &self.0
    }
}

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

impl<Nulls: Nullability, Storage: Buffer> CollectionAllocIn for Boolean<Nulls, Storage>
where
    Nulls::Collection<Bitmap<Storage>, Storage>: CollectionAllocIn,
{
    type Alloc = <Nulls::Collection<Bitmap<Storage>, Storage> as CollectionAllocIn>::Alloc;

    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self {
        Self(Nulls::Collection::with_capacity_in(capacity, alloc))
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, alloc: Self::Alloc) -> Self {
        Self(Nulls::Collection::from_iter_in(iter, alloc))
    }

    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, AllocError> {
        Nulls::Collection::try_with_capacity_in(capacity, alloc).map(Self)
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, AllocError> {
        Nulls::Collection::try_from_iter_in(iter, alloc).map(Self)
    }
}

impl<Nulls: Nullability, Storage: Buffer> CollectionRealloc for Boolean<Nulls, Storage>
where
    Nulls::Collection<Bitmap<Storage>, Storage>: CollectionRealloc,
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
    use crate::{collection::tests::round_trip, nullability::Nullable};

    use super::*;

    #[test]
    fn from_buffer() {
        let boolean = [true, false, true, true]
            .into_iter()
            .collect::<Boolean<NonNullable>>();
        let restored = Boolean::<NonNullable>::from_buffer(boolean.into_buffer());
        assert_eq!(restored.len(), 4);
        assert_eq!(restored.owned(0), Some(true));
        assert_eq!(restored.owned(1), Some(false));
    }

    #[test]
    fn collection() {
        round_trip::<Boolean, _>([true, false, true, true]);
        round_trip::<Boolean<Nullable>, _>([Some(true), None, Some(false), Some(true)]);
    }
}
