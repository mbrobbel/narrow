//! Nullable data with a validity bitmap.

use core::{
    borrow::BorrowMut,
    fmt::{self, Debug},
    iter::{self, Map, Zip},
};

use crate::{
    bitmap::Bitmap,
    buffer::{Buffer, VecBuffer},
    collection::{Collection, CollectionAlloc, CollectionRealloc, view::AsView},
    length::Length,
};

/// Nullable data with a validity bitmap.
///
/// Store a [`Collection`] `T` with a validity [`Bitmap`] that indicates the
/// validity (non-nullness) or invalidity (nullness) of items in the
/// collection.
///
/// `Storage` is the [`Buffer`] of the [`Bitmap`].
/// A panicking extension leaves committed chunks visible. The next extension
/// discards any uncommitted suffix.
pub struct Validity<T: Collection, Storage: Buffer = VecBuffer> {
    /// Collection that may contain null elements.
    collection: T,

    /// The validity bitmap with validity information for the items in the
    /// data.
    bitmap: Bitmap<Storage>,
}

impl<T: Collection + Debug, Storage: Buffer> Debug for Validity<T, Storage>
where
    Bitmap<Storage>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Validity")
            .field("collection", &self.collection)
            .field("bitmap", &self.bitmap)
            .finish()
    }
}

impl<T: Default + Collection, Storage: Buffer<For<u8>: Default>> Default for Validity<T, Storage> {
    fn default() -> Self {
        Self {
            collection: Default::default(),
            bitmap: Bitmap::default(),
        }
    }
}

impl<'collection, T: AsView<'collection>> AsView<'collection> for Option<T> {
    type View = Option<<T as AsView<'collection>>::View>;
    fn as_view(&'collection self) -> Option<<T as AsView<'collection>>::View> {
        self.as_ref().map(AsView::as_view)
    }
}

impl<T: Collection, Storage: Buffer> Length for Validity<T, Storage> {
    fn len(&self) -> usize {
        self.bitmap.len()
    }
}

impl<'collection, T: Collection, Storage: Buffer> IntoIterator
    for &'collection Validity<T, Storage>
{
    type Item = Option<<T as Collection>::View<'collection>>;
    type IntoIter = Map<
        Zip<
            <Bitmap<Storage> as Collection>::Iter<'collection>,
            <T as Collection>::Iter<'collection>,
        >,
        fn(
            (bool, <T as Collection>::View<'collection>),
        ) -> Option<<T as Collection>::View<'collection>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.bitmap
            .iter_views()
            .zip(self.collection.iter_views())
            .map(|(validity, value)| validity.then_some(value))
    }
}

impl<T: Collection, Storage: Buffer> IntoIterator for Validity<T, Storage> {
    type Item = Option<<T as Collection>::Owned>;

    type IntoIter = Map<
        Zip<<Bitmap<Storage> as Collection>::IntoIter, <T as Collection>::IntoIter>,
        fn((bool, <T as Collection>::Owned)) -> Option<<T as Collection>::Owned>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.bitmap
            .into_iter_owned()
            .zip(self.collection.into_iter_owned())
            .map(|(validity, value)| validity.then_some(value))
    }
}

impl<T: Collection, Storage: Buffer> Collection for Validity<T, Storage> {
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

    fn iter_views(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }

    type IntoIter = <Self as IntoIterator>::IntoIter;

    fn into_iter_owned(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<
    U: Default,
    T: CollectionAlloc<Owned = U>,
    Storage: Buffer<For<u8>: BorrowMut<[u8]> + CollectionRealloc>,
> FromIterator<Option<U>> for Validity<T, Storage>
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

/// Number of items buffered before flushing their validity to the bitmap in
/// [`Extend::extend`] for [`Validity`].
const VALIDITY_CHUNK: usize = 1024;

impl<
    U: Default,
    T: CollectionRealloc<Owned = U>,
    Storage: Buffer<For<u8>: BorrowMut<[u8]> + CollectionRealloc>,
> Extend<Option<U>> for Validity<T, Storage>
{
    fn extend<I: IntoIterator<Item = Option<U>>>(&mut self, iter: I) {
        self.collection.truncate(self.bitmap.len());

        // Buffer validity and flush it to the bitmap per chunk, instead of
        // extending the bitmap bit by bit: bulk bitmap extension uses the bit
        // packing fast path. The bitmap is extended after the items of a
        // chunk are committed to the collection, so a panicking iterator can
        // not add validity for items that never arrive.
        let mut items = iter.into_iter();
        loop {
            let mut validity = [false; VALIDITY_CHUNK];
            let mut count: usize = 0;
            self.collection.extend(
                items
                    .by_ref()
                    .take(VALIDITY_CHUNK)
                    .inspect(|opt| {
                        validity[count] = opt.is_some();
                        count = count.strict_add(1);
                    })
                    .map(Option::unwrap_or_default),
            );
            self.bitmap.extend(validity.iter().take(count));
            if count < VALIDITY_CHUNK {
                break;
            }
        }
    }
}

impl<
    T: CollectionRealloc<Owned: Default>,
    Storage: Buffer<For<u8>: BorrowMut<[u8]> + CollectionRealloc>,
> CollectionAlloc for Validity<T, Storage>
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
    Storage: Buffer<For<u8>: BorrowMut<[u8]> + CollectionRealloc>,
> CollectionRealloc for Validity<T, Storage>
{
    fn reserve(&mut self, additional: usize) {
        self.bitmap.reserve(additional);
        self.collection.reserve(additional);
    }

    fn truncate(&mut self, len: usize) {
        self.bitmap.truncate(len);
        self.collection.truncate(len);
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec::Vec;

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
        assert_eq!(validity.iter_views().collect::<Vec<_>>(), input);
        assert_eq!(validity.into_iter_owned().collect::<Vec<_>>(), input);
    }

    #[test]
    fn from_iter_nested() {
        let input = [Some(Some(1)), None, Some(None), Some(Some(4))];
        let validity = IntoIterator::into_iter(input).collect::<Validity<Vec<_>>>();
        assert_eq!(validity.len(), 4);
        assert_eq!(validity.iter_views().collect::<Vec<_>>(), input);
        assert_eq!(validity.into_iter_owned().collect::<Vec<_>>(), input);
    }

    #[test]
    fn iter() {
        let input = [Some(1), None, Some(3), Some(4)];
        let validity = IntoIterator::into_iter(input).collect::<Validity<Vec<_>>>();
        assert_eq!(validity.len(), 4);
        assert_eq!(Collection::iter_views(&validity).collect::<Vec<_>>(), input);
    }

    #[test]
    fn truncate() {
        let mut validity = [Some(1), None, Some(3), Some(4)]
            .into_iter()
            .collect::<Validity<Vec<_>>>();
        validity.truncate(2);
        assert_eq!(validity.len(), 2);
        assert_eq!(validity.view(0), Some(Some(1)));
        assert_eq!(validity.view(1), Some(None));
        assert_eq!(validity.view(2), None);
    }

    #[test]
    fn extend() {
        let input = [Some(1), None, Some(3), Some(4)];
        let mut validity = IntoIterator::into_iter(input).collect::<Validity<Vec<_>>>();
        assert_eq!(validity.len(), 4);
        validity.extend([Some(5)]);
        assert_eq!(validity.len(), 5);
        assert_eq!(
            validity.iter_views().collect::<Vec<_>>(),
            [Some(1), None, Some(3), Some(4), Some(5)]
        );
    }

    #[test]
    fn from_iter_across_chunks() {
        let input = (0..2500_u32)
            .map(|index| (index % 3 != 0).then_some(index))
            .collect::<Vec<_>>();
        let validity = input.clone().into_iter().collect::<Validity<Vec<_>>>();
        assert_eq!(validity.len(), input.len());
        assert_eq!(validity.iter_views().collect::<Vec<_>>(), input);
    }

    #[test]
    fn extend_across_chunks() {
        let input = (0..2500_u32)
            .map(|index| (index % 3 != 0).then_some(index))
            .collect::<Vec<_>>();
        let mut validity = Validity::<Vec<u32>>::default();
        validity.extend(input.clone());
        assert_eq!(validity.len(), input.len());
        assert_eq!(validity.iter_views().collect::<Vec<_>>(), input);
    }

    #[test]
    fn extend_panic_discards_uncommitted_suffix() {
        extern crate std;
        use std::panic::{AssertUnwindSafe, catch_unwind};

        let mut validity = IntoIterator::into_iter([Some(1), None]).collect::<Validity<Vec<_>>>();
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                validity.extend(
                    [Some(3), Some(4)]
                        .into_iter()
                        .chain(iter::once_with(|| panic!("boom"))),
                );
            }))
            .is_err()
        );

        // Only the prefix committed before this extension remains visible.
        assert_eq!(validity.len(), 2);
        assert_eq!(validity.view(2), None);

        validity.extend([Some(5)]);
        assert_eq!(
            validity.iter_views().collect::<Vec<_>>(),
            [Some(1), None, Some(5)]
        );
    }

    #[test]
    fn extend_panic_preserves_committed_chunks() {
        extern crate std;
        use std::panic::{AssertUnwindSafe, catch_unwind};

        let mut validity = Validity::<Vec<usize>>::default();
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                validity.extend(
                    (0..(VALIDITY_CHUNK + 2))
                        .map(Some)
                        .chain(iter::once_with(|| panic!("boom"))),
                );
            }))
            .is_err()
        );

        assert_eq!(validity.len(), VALIDITY_CHUNK);
        assert_eq!(validity.view(0), Some(Some(0)));
        assert_eq!(
            validity.view(VALIDITY_CHUNK - 1),
            Some(Some(VALIDITY_CHUNK - 1))
        );

        validity.extend([Some(VALIDITY_CHUNK + 3)]);
        assert_eq!(validity.len(), VALIDITY_CHUNK + 1);
        assert_eq!(
            validity.view(VALIDITY_CHUNK),
            Some(Some(VALIDITY_CHUNK + 3))
        );
    }

    #[test]
    fn extend_panic_discards_nested_suffix() {
        extern crate std;
        use std::panic::{AssertUnwindSafe, catch_unwind};

        let mut validity = Validity::<Validity<Vec<u32>>>::default();
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                validity.extend(
                    [Some(Some(1)), Some(Some(2))]
                        .into_iter()
                        .chain(iter::once_with(|| panic!("boom"))),
                );
            }))
            .is_err()
        );

        assert_eq!(validity.len(), 0);
        validity.extend([Some(Some(3))]);
        assert_eq!(validity.iter_views().collect::<Vec<_>>(), [Some(Some(3))]);
    }
}
