//! Nullable data with a validity bitmap.

use std::iter::{self, Map, Zip};

use crate::{
    bitmap::Bitmap,
    buffer::{BufferMut, BufferType, VecBuffer},
    collection::{AsView, Collection, CollectionAlloc, CollectionRealloc, ViewOf},
    length::Length,
};

/// Nullable data with a validity bitmap.
///
/// Store a [`Collection`] `T` with a validity [`Bitmap`] that indicates the
/// validity (non-nullness) or invalidity (nullness) of items in the
/// collection.
///
/// `Buffer` is the [`BufferType`] of the [`Bitmap`].
#[derive(Debug)]
pub struct Validity<T: Collection, Buffer: BufferType = VecBuffer> {
    /// Collection that may contain null elements.
    collection: T,

    /// The validity bitmap with validity information for the items in the
    /// data.
    bitmap: Bitmap<Buffer>,
}

impl<T: Default + Collection, Buffer: BufferType<Buffer<u8>: Default>> Default
    for Validity<T, Buffer>
{
    fn default() -> Self {
        Self {
            collection: Default::default(),
            bitmap: Bitmap::default(),
        }
    }
}

impl<'collection, T: AsView<'collection>> AsView<'collection> for Option<T> {
    type View = Option<ViewOf<'collection, T>>;
    fn as_view(&'collection self) -> Option<ViewOf<'collection, T>> {
        self.as_ref().map(AsView::as_view)
    }
}

impl<T: Collection, Buffer: BufferType> Length for Validity<T, Buffer> {
    fn len(&self) -> usize {
        self.bitmap.len()
    }
}

impl<'collection, T: Collection, Buffer: BufferType> IntoIterator
    for &'collection Validity<T, Buffer>
{
    type Item = Option<<T as Collection>::View<'collection>>;
    type IntoIter = Map<
        Zip<
            <Bitmap<Buffer> as Collection>::Iter<'collection>,
            <T as Collection>::Iter<'collection>,
        >,
        fn(
            (bool, <T as Collection>::View<'collection>),
        ) -> Option<<T as Collection>::View<'collection>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.bitmap
            .iter()
            .zip(self.collection.iter())
            .map(|(validity, value)| validity.then_some(value))
    }
}

impl<T: Collection, Buffer: BufferType> IntoIterator for Validity<T, Buffer> {
    type Item = Option<<T as Collection>::Owned>;

    type IntoIter = Map<
        Zip<<Bitmap<Buffer> as Collection>::IntoIter, <T as Collection>::IntoIter>,
        fn((bool, <T as Collection>::Owned)) -> Option<<T as Collection>::Owned>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        Collection::into_iter(self.bitmap)
            .zip(self.collection.into_iter())
            .map(|(validity, value)| validity.then_some(value))
    }
}

impl<T: Collection, Buffer: BufferType> Collection for Validity<T, Buffer> {
    type View<'collection>
        = Option<<T as Collection>::View<'collection>>
    where
        Self: 'collection;

    type Owned = Option<<T as Collection>::Owned>;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.bitmap.view(index).map(|validity| {
            if validity {
                self.collection.view(index)
            } else {
                None
            }
        })
    }

    type Iter<'collection>
        = <&'collection Self as IntoIterator>::IntoIter
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }

    type IntoIter = <Self as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<
    U: Default,
    T: CollectionAlloc<Owned = U>,
    Buffer: BufferType<Buffer<u8>: BufferMut<u8> + CollectionRealloc>,
> FromIterator<Option<U>> for Validity<T, Buffer>
{
    fn from_iter<I: IntoIterator<Item = Option<U>>>(iter: I) -> Self {
        let items = iter.into_iter();
        let (lower_bound, upper_bound) = items.size_hint();
        let mut bitmap = Bitmap::with_capacity(upper_bound.unwrap_or(lower_bound));
        let collection = items
            .inspect(|opt| bitmap.extend(iter::once(opt.is_some())))
            .map(Option::unwrap_or_default)
            .collect();
        Self { collection, bitmap }
    }
}

impl<
    U: Default,
    T: CollectionRealloc<Owned = U>,
    Buffer: BufferType<Buffer<u8>: BufferMut<u8> + CollectionRealloc>,
> Extend<Option<U>> for Validity<T, Buffer>
{
    fn extend<I: IntoIterator<Item = Option<U>>>(&mut self, iter: I) {
        self.collection.extend(
            iter.into_iter()
                .inspect(|opt| self.bitmap.extend(iter::once(opt.is_some())))
                .map(Option::unwrap_or_default),
        );
    }
}

impl<
    T: CollectionRealloc<Owned: Default>,
    Buffer: BufferType<Buffer<u8>: BufferMut<u8> + CollectionRealloc>,
> CollectionAlloc for Validity<T, Buffer>
{
    fn with_capacity(capacity: usize) -> Self {
        Self {
            collection: T::with_capacity(capacity),
            bitmap: Bitmap::with_capacity(capacity),
        }
    }
}

impl<
    T: CollectionRealloc<Owned: Default>,
    Buffer: BufferType<Buffer<u8>: BufferMut<u8> + CollectionRealloc>,
> CollectionRealloc for Validity<T, Buffer>
{
    fn reserve(&mut self, additional: usize) {
        self.bitmap.reserve(additional);
        self.collection.reserve(additional);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        let input = [Some(1), None, Some(3), Some(4)];
        let validity = IntoIterator::into_iter(input).collect::<Validity<Vec<_>>>();
        assert_eq!(validity.len(), 4);
        assert_eq!(validity.view(0), Some(Some(1)));
        assert_eq!(validity.view(1), Some(None));
        assert_eq!(validity.view(2), Some(Some(3)));
        assert_eq!(validity.view(3), Some(Some(4)));
        assert_eq!(validity.view(4), None);
        assert_eq!(validity.iter().collect::<Vec<_>>(), input);
        assert_eq!(Collection::into_iter(validity).collect::<Vec<_>>(), input);
    }

    #[test]
    fn from_iter_nested() {
        let input = [Some(Some(1)), None, Some(None), Some(Some(4))];
        let validity = IntoIterator::into_iter(input).collect::<Validity<Vec<_>>>();
        assert_eq!(validity.len(), 4);
        assert_eq!(validity.iter().collect::<Vec<_>>(), input);
        assert_eq!(Collection::into_iter(validity).collect::<Vec<_>>(), input);
    }

    #[test]
    fn iter() {
        let input = [Some(1), None, Some(3), Some(4)];
        let validity = IntoIterator::into_iter(input).collect::<Validity<Vec<_>>>();
        assert_eq!(validity.len(), 4);
        assert_eq!(Collection::iter(&validity).collect::<Vec<_>>(), input);
    }

    #[test]
    fn extend() {
        let input = [Some(1), None, Some(3), Some(4)];
        let mut validity = IntoIterator::into_iter(input).collect::<Validity<Vec<_>>>();
        assert_eq!(validity.len(), 4);
        validity.extend([Some(5)]);
        assert_eq!(validity.len(), 5);
        assert_eq!(
            validity.iter().collect::<Vec<_>>(),
            [Some(1), None, Some(3), Some(4), Some(5)]
        );
    }
}
