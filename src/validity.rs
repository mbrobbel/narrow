//! Nullable data with an optional validity bitmap.

use core::{
    borrow::BorrowMut,
    fmt::{self, Debug},
};

use crate::{
    bitmap::{Bitmap, BitmapBuilder, ValidityBitmap},
    buffer::{Buffer, VecBuffer},
    collection::{
        AllocError, ChildRef, Collection, CollectionAlloc, CollectionAllocIn, CollectionRealloc,
        view::AsView,
    },
    length::Length,
};

/// Nullable data with an optional validity bitmap.
///
/// Stores a [`Collection`] `T` and an optional validity [`Bitmap`]. An omitted
/// bitmap means every item in the collection is valid.
///
/// `Storage` is the [`Buffer`] of the [`Bitmap`].
/// A panicking extension leaves committed chunks visible. The next extension
/// discards any uncommitted suffix.
///
/// Arrow stores nullness separately from values instead of interleaving
/// `Option<T>` objects. Invalid positions still have a physical placeholder,
/// and the bitmap determines whether that value is visible:
///
/// ```text
/// values:   [v0, __, v2]
/// validity: [ 1,  0,  1]
/// ```
///
/// # Examples
///
/// ```
/// use narrow::{collection::Collection, validity::Validity};
///
/// let values = [Some(1), None].into_iter().collect::<Validity<Vec<i32>>>();
/// assert_eq!(values.owned(0), Some(Some(1)));
/// assert_eq!(values.owned(1), Some(None));
/// ```
pub struct Validity<T: Collection, Storage: Buffer = VecBuffer> {
    /// Collection that may contain null elements.
    collection: T,

    /// Explicit validity bitmap, or [`None`] when every item is valid.
    bitmap: Option<Bitmap<Storage>>,
}

/// Error returned by [`Validity::try_from_parts`].
///
/// The value and validity collections must stay row-aligned. Checking their
/// lengths at the raw-parts boundary makes that invariant available to every
/// subsequent collection operation.
///
/// # Examples
///
/// ```
/// use narrow::{bitmap::Bitmap, validity::{Validity, ValidityError}};
///
/// let result = Validity::try_from_parts(vec![1, 2], [true].into_iter().collect::<Bitmap>());
/// assert_eq!(result.unwrap_err(), ValidityError::LengthMismatch { collection: 2, bitmap: 1 });
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidityError {
    /// The collection and the validity bitmap have different lengths.
    LengthMismatch {
        /// The length of the collection.
        collection: usize,
        /// The length of the validity bitmap.
        bitmap: usize,
    },
}

impl fmt::Display for ValidityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::LengthMismatch { collection, bitmap } => write!(
                f,
                "collection length ({collection}) does not match bitmap length ({bitmap})"
            ),
        }
    }
}

impl core::error::Error for ValidityError {}

impl<T: Collection, Storage: Buffer> Validity<T, Storage> {
    /// Constructs a [`Validity`] from a `collection` and its validity `bitmap`.
    ///
    /// # Errors
    ///
    /// Returns a [`ValidityError`] when the length of the collection does not
    /// match the length of the bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{bitmap::Bitmap, collection::Collection, validity::Validity};
    ///
    /// let bitmap = [true, false].into_iter().collect::<Bitmap>();
    /// let values = Validity::try_from_parts(vec![1, 0], bitmap).unwrap();
    /// assert_eq!(values.owned(1), Some(None));
    /// ```
    pub fn try_from_parts(collection: T, bitmap: Bitmap<Storage>) -> Result<Self, ValidityError> {
        let (collection_len, bitmap_len) = (collection.len(), bitmap.len());
        if collection_len == bitmap_len {
            Ok(Self {
                collection,
                bitmap: Some(bitmap),
            })
        } else {
            Err(ValidityError::LengthMismatch {
                collection: collection_len,
                bitmap: bitmap_len,
            })
        }
    }

    /// Returns the collection and optional validity bitmap of this
    /// [`Validity`].
    ///
    /// The bitmap is [`None`] when all items are implicitly valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{bitmap::Bitmap, validity::Validity};
    ///
    /// let bitmap = [true, true].into_iter().collect::<Bitmap>();
    /// let values = Validity::try_from_parts(vec![1, 2], bitmap).unwrap();
    /// let (data, validity) = values.into_parts();
    /// assert_eq!(data, [1, 2]);
    /// assert_eq!(validity.unwrap().into_iter().collect::<Vec<_>>(), [true, true]);
    /// ```
    #[must_use]
    pub fn into_parts(self) -> (T, Option<Bitmap<Storage>>) {
        (self.collection, self.bitmap)
    }

    /// Constructs a [`Validity`] whose items are all valid without storing a
    /// bitmap.
    ///
    /// This represents an Arrow validity bitmap omitted for an array with no
    /// null items.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{bitmap::ValidityBitmap, validity::Validity};
    ///
    /// let values = Validity::<Vec<i32>>::from_collection(vec![1, 2]);
    /// assert!(values.bitmap_ref().is_none());
    /// assert!(values.all_valid());
    /// ```
    #[must_use]
    pub fn from_collection(collection: T) -> Self {
        Self {
            collection,
            bitmap: None,
        }
    }
}

impl<T: Collection, Storage: Buffer> ChildRef for Validity<T, Storage> {
    type Child = T;

    fn child_ref(&self) -> &Self::Child {
        &self.collection
    }
}

impl<T: Collection, Storage: Buffer> ValidityBitmap for Validity<T, Storage> {
    type Storage = Storage;

    fn bitmap_ref(&self) -> Option<&Bitmap<Self::Storage>> {
        self.bitmap.as_ref()
    }
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

impl<T: Default + Collection, Storage: Buffer> Default for Validity<T, Storage> {
    fn default() -> Self {
        Self {
            collection: Default::default(),
            bitmap: None,
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
        self.bitmap
            .as_ref()
            .map_or_else(|| self.collection.len(), Length::len)
    }
}

/// Iterator over nullable values.
#[derive(Debug)]
pub struct ValidityIter<Values, Bits> {
    /// Values returned by the iterator.
    values: Values,
    /// Explicit validity bits, or [`None`] when every value is valid.
    bits: Option<Bits>,
}

impl<Value, Values, Bits> Iterator for ValidityIter<Values, Bits>
where
    Values: Iterator<Item = Value>,
    Bits: Iterator<Item = bool>,
{
    type Item = Option<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        let valid = match self.bits.as_mut() {
            Some(bits) => bits.next()?,
            None => true,
        };
        self.values.next().map(|value| valid.then_some(value))
    }

    /// Drives the validity bits with their own (fast) `fold`, pulling one
    /// value per bit, instead of calling `next` on both iterators per item.
    fn fold<B, F: FnMut(B, Self::Item) -> B>(self, init: B, mut f: F) -> B {
        let Self {
            mut values,
            bits: validity,
        } = self;
        match validity {
            None => values.fold(init, |acc, value| f(acc, Some(value))),
            Some(bits) => {
                // Like `next`, stop at the first missing value, even if the
                // values iterator is not fused: once exhausted, stay exhausted.
                bits.fold((init, false), |(acc, exhausted), valid| match (!exhausted)
                    .then(|| values.next())
                    .flatten()
                {
                    Some(value) => (f(acc, valid.then_some(value)), false),
                    None => (acc, true),
                })
                .0
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let values = self.values.size_hint();
        self.bits.as_ref().map_or(values, |bits_iter| {
            let bit_hint = bits_iter.size_hint();
            (
                values.0.min(bit_hint.0),
                values
                    .1
                    .zip(bit_hint.1)
                    .map(|(value_count, bit_count)| value_count.min(bit_count)),
            )
        })
    }
}

impl<Value, Values, Bits> ExactSizeIterator for ValidityIter<Values, Bits>
where
    Values: ExactSizeIterator<Item = Value>,
    Bits: ExactSizeIterator<Item = bool>,
{
    fn len(&self) -> usize {
        self.bits.as_ref().map_or_else(
            || self.values.len(),
            |bits| self.values.len().min(bits.len()),
        )
    }
}

impl<'collection, T: Collection, Storage: Buffer> IntoIterator
    for &'collection Validity<T, Storage>
{
    type Item = Option<<T as Collection>::View<'collection>>;
    type IntoIter = ValidityIter<
        <T as Collection>::Iter<'collection>,
        <Bitmap<Storage> as Collection>::Iter<'collection>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        ValidityIter {
            values: self.collection.iter_views(),
            bits: self.bitmap.as_ref().map(IntoIterator::into_iter),
        }
    }
}

impl<T: Collection, Storage: Buffer> IntoIterator for Validity<T, Storage> {
    type Item = Option<<T as Collection>::Owned>;

    type IntoIter =
        ValidityIter<<T as Collection>::IntoIter, <Bitmap<Storage> as Collection>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        ValidityIter {
            values: self.collection.into_iter_owned(),
            bits: self.bitmap.map(Collection::into_iter_owned),
        }
    }
}

impl<T: Collection, Storage: Buffer> Collection for Validity<T, Storage> {
    type View<'collection>
        = Option<<T as Collection>::View<'collection>>
    where
        Self: 'collection;

    type Owned = Option<<T as Collection>::Owned>;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        // The bitmap bounds the length when present: a panicking extension may
        // leave uncommitted values in the collection beyond it.
        let value = self.collection.view(index)?;
        self.bitmap_ref().map_or(Some(Some(value)), |bitmap| {
            bitmap.view(index).map(|valid| valid.then_some(value))
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
    T: CollectionRealloc,
    Storage: Buffer<For<u8>: BorrowMut<[u8]> + CollectionRealloc<Alloc = T::Alloc>>,
> Validity<T, Storage>
{
    /// Returns the collection and its bitmap, first materializing an
    /// explicit all-valid bitmap with the collection's allocator if there is
    /// none.
    fn materialized_mut(&mut self) -> (&mut T, &mut Bitmap<Storage>) {
        let &mut Self {
            ref mut collection,
            ref mut bitmap,
        } = self;
        let materialized = bitmap.get_or_insert_with(|| {
            Bitmap::<Storage>::all_set_in(collection.len(), collection.allocator())
        });
        (collection, materialized)
    }

    /// Returns the collection and its bitmap, first trying to materialize an
    /// explicit all-valid bitmap with the collection's allocator if there is
    /// none.
    fn try_materialized_mut(&mut self) -> Result<(&mut T, &mut Bitmap<Storage>), AllocError> {
        let &mut Self {
            ref mut collection,
            ref mut bitmap,
        } = self;
        let materialized = bitmap.take().map_or_else(
            || Bitmap::<Storage>::try_all_set_in(collection.len(), collection.allocator()),
            Ok,
        )?;
        Ok((collection, bitmap.insert(materialized)))
    }
}

impl<
    U: Default,
    T: CollectionAlloc<Owned = U>,
    Storage: Buffer<For<u8>: BorrowMut<[u8]> + CollectionAlloc + CollectionRealloc>,
> FromIterator<Option<U>> for Validity<T, Storage>
{
    fn from_iter<I: IntoIterator<Item = Option<U>>>(iter: I) -> Self {
        let items = iter.into_iter();
        let (lower_bound, upper_bound) = items.size_hint();
        let mut validity = BitmapBuilder::<Storage>::with_capacity_in(
            upper_bound.unwrap_or(lower_bound),
            Default::default(),
        );
        let collection = items
            .inspect(|opt| validity.push(opt.is_some()))
            .map(Option::unwrap_or_default)
            .collect();
        Self {
            collection,
            bitmap: Some(validity.finish()),
        }
    }
}

/// Number of items buffered before flushing their validity to the bitmap in
/// [`Extend::extend`] for [`Validity`].
const VALIDITY_CHUNK: usize = 1024;

impl<
    U: Default,
    T: CollectionRealloc<Owned = U>,
    Storage: Buffer<For<u8>: BorrowMut<[u8]> + CollectionRealloc<Alloc = T::Alloc>>,
> Extend<Option<U>> for Validity<T, Storage>
{
    fn extend<I: IntoIterator<Item = Option<U>>>(&mut self, iter: I) {
        let mut items = iter.into_iter();
        let (collection, bitmap) = self.materialized_mut();
        collection.truncate(bitmap.len());

        // Reserve for the known minimum up front: growing both buffers chunk
        // by chunk would reallocate and copy them repeatedly.
        let additional = items.size_hint().0;
        bitmap.reserve(additional);
        collection.reserve(additional);

        // Buffer validity and flush it to the bitmap per chunk, instead of
        // extending the bitmap bit by bit: bulk bitmap extension uses the bit
        // packing fast path. The bitmap is extended after the items of a
        // chunk are committed to the collection, so a panicking iterator can
        // not add validity for items that never arrive.
        loop {
            let mut validity = [false; VALIDITY_CHUNK];
            let mut count: usize = 0;
            collection.extend(
                items
                    .by_ref()
                    .take(VALIDITY_CHUNK)
                    .inspect(|opt| {
                        validity[count] = opt.is_some();
                        count = count.strict_add(1);
                    })
                    .map(Option::unwrap_or_default),
            );
            bitmap.extend(validity.iter().take(count));
            if count < VALIDITY_CHUNK {
                break;
            }
        }
    }
}

impl<
    T: CollectionRealloc<Owned: Default>,
    Storage: Buffer<For<u8>: BorrowMut<[u8]> + CollectionRealloc<Alloc = T::Alloc>>,
> CollectionAllocIn for Validity<T, Storage>
{
    type Alloc = T::Alloc;

    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self {
        let collection = T::with_capacity_in(capacity, alloc.clone());
        let bitmap = Bitmap::<Storage>::with_capacity_in(capacity, alloc);
        Self {
            collection,
            bitmap: Some(bitmap),
        }
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, alloc: Self::Alloc) -> Self {
        let items = iter.into_iter();
        let (lower_bound, upper_bound) = items.size_hint();
        let mut validity = BitmapBuilder::<Storage>::with_capacity_in(
            upper_bound.unwrap_or(lower_bound),
            alloc.clone(),
        );
        let collection = T::from_iter_in(
            items
                .inspect(|item| validity.push(item.is_some()))
                .map(Option::unwrap_or_default),
            alloc,
        );
        Self {
            collection,
            bitmap: Some(validity.finish()),
        }
    }

    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, AllocError> {
        let collection = T::try_with_capacity_in(capacity, alloc.clone())?;
        let bitmap = Bitmap::<Storage>::try_with_capacity_in(capacity, alloc)?;
        Ok(Self {
            collection,
            bitmap: Some(bitmap),
        })
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, AllocError> {
        let items = iter.into_iter();
        let (lower_bound, upper_bound) = items.size_hint();
        let mut validity = Self::try_with_capacity_in(upper_bound.unwrap_or(lower_bound), alloc)?;
        validity.try_extend(items)?;
        Ok(validity)
    }
}

impl<
    T: CollectionRealloc<Owned: Default>,
    Storage: Buffer<For<u8>: BorrowMut<[u8]> + CollectionRealloc<Alloc = T::Alloc>>,
> CollectionRealloc for Validity<T, Storage>
{
    fn allocator(&self) -> Self::Alloc {
        self.collection.allocator()
    }

    fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError> {
        let (collection, bitmap) = self.try_materialized_mut()?;
        collection.try_reserve(additional)?;
        bitmap.try_reserve(additional)
    }

    fn try_extend<I: IntoIterator<Item = Self::Owned>>(
        &mut self,
        iter: I,
    ) -> Result<(), AllocError> {
        let len = self.len();
        let mut items = iter.into_iter();
        let (collection, bitmap) = self.try_materialized_mut()?;
        collection.truncate(len);

        // Reserve for the known minimum up front: growing both buffers chunk
        // by chunk would reallocate and copy them repeatedly.
        let additional = items.size_hint().0;
        collection.try_reserve(additional)?;
        bitmap.try_reserve(additional)?;
        loop {
            let mut validity = [false; VALIDITY_CHUNK];
            let mut count: usize = 0;
            let values = items
                .by_ref()
                .take(VALIDITY_CHUNK)
                .inspect(|item| {
                    validity[count] = item.is_some();
                    count = count.strict_add(1);
                })
                .map(Option::unwrap_or_default);
            let extended = collection
                .try_extend(values)
                .and_then(|()| bitmap.try_extend(validity.into_iter().take(count)));
            if let Err(error) = extended {
                bitmap.truncate(len);
                collection.truncate(len);
                return Err(error);
            }
            if count < VALIDITY_CHUNK {
                break;
            }
        }
        Ok(())
    }

    fn reserve(&mut self, additional: usize) {
        let (collection, bitmap) = self.materialized_mut();
        bitmap.reserve(additional);
        collection.reserve(additional);
    }

    fn truncate(&mut self, len: usize) {
        if let Some(bitmap) = self.bitmap.as_mut() {
            bitmap.truncate(len);
        }
        self.collection.truncate(len);
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec::Vec;
    use core::{
        borrow::{Borrow, BorrowMut},
        iter, slice,
    };

    use super::*;
    use crate::fixed_size::FixedSize;

    #[test]
    fn iter_fold_matches_next() {
        let bits = [
            true, false, true, true, false, false, true, true, true, false,
        ];
        for values_len in 0..=12 {
            for bits_len in 0..=bits.len() {
                let explicit = Bitmap::<VecBuffer>::from_iter(&bits[..bits_len]);
                for bitmap in [Some(&explicit), None] {
                    let make = || ValidityIter {
                        values: 0..values_len,
                        bits: bitmap.map(IntoIterator::into_iter),
                    };
                    // A `for` loop drives the iterator with `next`.
                    let mut expected = Vec::new();
                    for item in make() {
                        expected.push(item);
                    }
                    let folded = make().fold(Vec::new(), |mut acc, item| {
                        acc.push(item);
                        acc
                    });
                    assert_eq!(folded, expected);
                    assert_eq!(make().flatten().count(), expected.iter().flatten().count());
                }
            }
        }
    }

    #[test]
    fn iter_fold_stops_at_first_missing_value() {
        // A non-fused values iterator that yields again after `None`.
        let mut source = [Some(1), None, Some(2)].into_iter();
        let values = iter::from_fn(move || source.next().flatten());
        let bitmap = Bitmap::<VecBuffer>::from_iter([true; 3]);
        let folded = ValidityIter {
            values,
            bits: Some(bitmap.iter_views()),
        }
        .fold(Vec::new(), |mut acc, item| {
            acc.push(item);
            acc
        });
        assert_eq!(folded, [Some(1)]);
    }

    /// Non-default allocator handle used to verify lazy bitmap allocation.
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestAllocator(u8);

    /// Buffer whose collections retain a [`TestAllocator`].
    #[derive(Clone, Copy, Debug, Default)]
    struct TestBuffer;

    impl Buffer for TestBuffer {
        type For<T: FixedSize> = TestCollection<T>;
    }

    /// Growable test collection with an explicit allocator handle.
    #[derive(Debug)]
    struct TestCollection<T> {
        /// Stored values.
        values: Vec<T>,
        /// Allocator used to create the collection.
        allocator: TestAllocator,
    }

    impl<T> Borrow<[T]> for TestCollection<T> {
        fn borrow(&self) -> &[T] {
            &self.values
        }
    }

    impl<T> BorrowMut<[T]> for TestCollection<T> {
        fn borrow_mut(&mut self) -> &mut [T] {
            &mut self.values
        }
    }

    impl<T: FixedSize> Collection for TestCollection<T> {
        type View<'collection>
            = T
        where
            Self: 'collection;
        type Owned = T;

        fn view(&self, index: usize) -> Option<Self::View<'_>> {
            self.values.get(index).copied()
        }

        type Iter<'collection>
            = core::iter::Copied<slice::Iter<'collection, T>>
        where
            Self: 'collection;

        fn iter_views(&self) -> Self::Iter<'_> {
            self.values.iter().copied()
        }

        type IntoIter = alloc::vec::IntoIter<T>;

        fn into_iter_owned(self) -> Self::IntoIter {
            self.values.into_iter()
        }
    }

    impl<T> Length for TestCollection<T> {
        fn len(&self) -> usize {
            self.values.len()
        }
    }

    impl<T: FixedSize> CollectionAllocIn for TestCollection<T> {
        type Alloc = TestAllocator;

        fn with_capacity_in(capacity: usize, allocator: Self::Alloc) -> Self {
            Self {
                values: Vec::with_capacity(capacity),
                allocator,
            }
        }

        fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(
            iter: I,
            allocator: Self::Alloc,
        ) -> Self {
            Self {
                values: iter.into_iter().collect(),
                allocator,
            }
        }

        fn try_with_capacity_in(
            capacity: usize,
            allocator: Self::Alloc,
        ) -> Result<Self, AllocError> {
            let mut values = Vec::new();
            values.try_reserve_exact(capacity).map_err(|_| AllocError)?;
            Ok(Self { values, allocator })
        }

        fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
            iter: I,
            allocator: Self::Alloc,
        ) -> Result<Self, AllocError> {
            let items = iter.into_iter();
            let (capacity, _) = items.size_hint();
            let mut collection = Self::try_with_capacity_in(capacity, allocator)?;
            collection.try_extend(items)?;
            Ok(collection)
        }
    }

    impl<T: FixedSize> Extend<T> for TestCollection<T> {
        fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
            self.values.extend(iter);
        }
    }

    impl<T: FixedSize> CollectionRealloc for TestCollection<T> {
        fn allocator(&self) -> Self::Alloc {
            self.allocator.clone()
        }

        fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError> {
            self.values.try_reserve(additional).map_err(|_| AllocError)
        }

        fn try_extend<I: IntoIterator<Item = Self::Owned>>(
            &mut self,
            iter: I,
        ) -> Result<(), AllocError> {
            let items = iter.into_iter();
            let (additional, _) = items.size_hint();
            self.try_reserve(additional)?;
            self.values.extend(items);
            Ok(())
        }

        fn reserve(&mut self, additional: usize) {
            self.values.reserve(additional);
        }

        fn truncate(&mut self, len: usize) {
            self.values.truncate(len);
        }
    }

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
    fn try_from_parts() {
        let collection = alloc::vec![1, 0, 3, 4];
        let bitmap = Bitmap::<VecBuffer>::from_iter([true, false, true, true]);
        let validity = Validity::try_from_parts(collection, bitmap).expect("valid parts");
        assert_eq!(validity.len(), 4);
        assert_eq!(validity.view(0), Some(Some(1)));
        assert_eq!(validity.view(1), Some(None));
        let (values, restored) = validity.into_parts();
        assert_eq!(values, alloc::vec![1, 0, 3, 4]);
        assert_eq!(restored.expect("explicit bitmap").len(), 4);
    }

    #[test]
    fn implicit_all_valid() {
        let validity = Validity::<Vec<i32>>::from_collection(alloc::vec![1, 2, 3]);
        assert_eq!(validity.len(), 3);
        assert!(validity.bitmap_ref().is_none());
        assert_eq!(
            validity.iter_views().collect::<Vec<_>>(),
            [Some(1), Some(2), Some(3)]
        );
        assert!(validity.into_parts().1.is_none());
    }

    #[test]
    fn extend_materializes_implicit_validity() {
        let mut validity = Validity::<Vec<i32>>::from_collection(alloc::vec![1, 2]);
        validity.extend([None, Some(4)]);

        assert!(validity.bitmap_ref().is_some());
        assert_eq!(
            validity.iter_views().collect::<Vec<_>>(),
            [Some(1), Some(2), None, Some(4)]
        );
    }

    #[test]
    fn materializes_with_the_collection_allocator() {
        let allocator = TestAllocator(7);
        let collection = TestCollection::from_iter_in([1_i32, 2], allocator.clone());
        let mut validity = Validity::<_, TestBuffer>::from_collection(collection);

        validity.extend([None, Some(4)]);

        let (_, bitmap) = validity.into_parts();
        assert_eq!(bitmap.expect("materialized bitmap").allocator(), allocator);
    }

    #[test]
    fn try_from_parts_length_mismatch() {
        let bitmap = Bitmap::<VecBuffer>::from_iter([true, false]);
        let error = Validity::<Vec<i32>>::try_from_parts(alloc::vec![1, 2, 3], bitmap)
            .expect_err("length mismatch");
        assert_eq!(
            error,
            ValidityError::LengthMismatch {
                collection: 3,
                bitmap: 2,
            }
        );
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
