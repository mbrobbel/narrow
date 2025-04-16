//! Offsets for variable-sized arrays.

use crate::{
    FixedSize, Index, Length,
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{Buffer, BufferType, VecBuffer},
    nullability::{NonNullable, Nullability, Nullable},
    validity::Validity,
};
use std::{
    iter::{self, Map, Peekable, Zip},
    num::TryFromIntError,
    ops::{AddAssign, Range, Sub},
};

/// Types representing offset values.
///
/// Values with these types can be used to represent offset values.
///
/// This trait is sealed to prevent downstream implementations.
pub trait Offset:
    FixedSize
    + AddAssign
    + Default
    + TryFrom<usize, Error = TryFromIntError>
    + TryInto<usize, Error = TryFromIntError>
    + Sub<Output = Self>
    + sealed::Sealed
    + 'static
{
    /// The unsigned variant of Self.
    type Unsigned: TryFrom<usize, Error = TryFromIntError>;

    /// Checked addition. Computes `self + rhs`, returning `None` if overflow occurred.
    fn checked_add(self, rhs: Self) -> Option<Self>;

    /// Checked addition with an unsigned value. Computes `self + rhs`, returning `None` if overflow occurred.
    fn checked_add_unsigned(self, rhs: Self::Unsigned) -> Option<Self>;
}

/// Indicates that an [`Offset`] generic is not applicable.
///
/// This is used instead to prevent confusion in code because we don't have default
/// types for generic associated types.
///
/// This still shows up as [`i32`] in documentation but there is no way
/// to prevent that.
pub type NA = i32;

/// Private module for a seal trait.
mod sealed {
    /// Sealed trait to seal [`super::Offset`].
    pub trait Sealed {}

    impl<T> Sealed for T where T: super::Offset {}
}

impl Offset for i32 {
    type Unsigned = u32;

    fn checked_add(self, rhs: Self) -> Option<Self> {
        i32::checked_add(self, rhs)
    }

    fn checked_add_unsigned(self, rhs: Self::Unsigned) -> Option<Self> {
        i32::checked_add_unsigned(self, rhs)
    }
}

impl Offset for i64 {
    type Unsigned = u64;

    fn checked_add(self, rhs: Self) -> Option<Self> {
        i64::checked_add(self, rhs)
    }

    fn checked_add_unsigned(self, rhs: Self::Unsigned) -> Option<Self> {
        i64::checked_add_unsigned(self, rhs)
    }
}

/// A reference to a slot in an offset
#[allow(unused)]
pub struct OffsetSlot<'a, OffsetItem: Offset, Buffer: BufferType> {
    /// The offset buffer
    offset: &'a <Buffer as BufferType>::Buffer<OffsetItem>,
    /// The position in the offset
    index: usize,
}

impl<OffsetItem: Offset, Buffer: BufferType> OffsetSlot<'_, OffsetItem, Buffer> {
    /// Returns the position of this slot in the buffer i.e. the index.
    #[must_use]
    pub fn position(&self) -> usize {
        self.index
    }

    /// Returns the start index of this offset slot.
    #[must_use]
    pub fn start(&self) -> OffsetItem {
        // Safety:
        // - The index of an offset slot is valid by construction.
        unsafe {
            self.offset
                .as_slice()
                .index_unchecked(self.index)
                .to_owned()
        }
    }

    /// Returns the start index of this offset slot as usize.
    ///
    /// # Panics
    ///
    /// This function panics if the conversion of [`Offset`] to [`usize`] fails.
    #[must_use]
    pub fn start_usize(&self) -> usize {
        self.start().try_into().expect("convert fail")
    }

    /// Returns this offset as [`Range`].
    #[must_use]
    pub fn range(&self) -> Range<OffsetItem> {
        self.start()..self.end()
    }

    /// Returns this offset as [`Range`] of usize.
    #[must_use]
    pub fn range_usize(&self) -> Range<usize> {
        self.start_usize()..self.end_usize()
    }

    /// Returns the end index of this offset slot.
    #[must_use]
    pub fn end(&self) -> OffsetItem {
        // Safety:
        // - The index (+1) of an offset slot is valid by construction.
        unsafe {
            self.offset
                .as_slice()
                .index_unchecked(self.index + 1)
                .to_owned()
        }
    }

    /// Returns the end index of this offset slot as usize.
    /// # Panics
    ///
    /// This function panics if the conversion of [`Offset`] to [`usize`] fails.
    #[must_use]
    pub fn end_usize(&self) -> usize {
        self.end().try_into().expect("convert fail")
    }

    /// Returns the length of this offset slot.
    #[must_use]
    pub fn len(&self) -> OffsetItem {
        self.end() - self.start()
    }

    /// Returns the length of this offset slot as usize.
    ///
    /// # Panics
    ///
    /// This function panics if the conversion of [`Offset`] to [`usize`] fails.
    #[must_use]
    pub fn len_usize(&self) -> usize {
        self.len().try_into().expect("convert fail")
    }

    /// Returns the start and end index of this slot as tuple.
    #[must_use]
    pub fn tuple(&self) -> (OffsetItem, OffsetItem) {
        (self.start(), self.end())
    }

    /// Returns the start and end index of this slot as usize tuple.
    #[must_use]
    pub fn tuple_usize(&self) -> (usize, usize) {
        (self.start_usize(), self.end_usize())
    }
}

/// Offsets abstraction.
pub struct Offsets<
    T,
    Nulls: Nullability = NonNullable,
    OffsetItem: Offset = i32,
    Buffer: BufferType = VecBuffer,
> {
    /// The data
    pub data: T,

    /// The offsets
    pub offsets: Nulls::Collection<Buffer::Buffer<OffsetItem>, Buffer>,
}

impl<T, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType>
    Offsets<T, Nulls, OffsetItem, Buffer>
where
    Offsets<T, Nulls, OffsetItem, Buffer>: Index,
{
    /// Returns an iteratover over the offset items in this [`Offsets`].
    pub fn iter(&self) -> OffsetIter<'_, Nulls, T, OffsetItem, Buffer> {
        <&Self as IntoIterator>::into_iter(self)
    }
}

impl<T, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType> Clone
    for Offsets<T, Nulls, OffsetItem, Buffer>
where
    T: Clone,
    Nulls::Collection<Buffer::Buffer<OffsetItem>, Buffer>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            offsets: self.offsets.clone(),
        }
    }
}

impl<T: Default, OffsetItem: Offset, Buffer> Default for Offsets<T, NonNullable, OffsetItem, Buffer>
where
    Buffer: BufferType<Buffer<OffsetItem>: Default + Extend<OffsetItem>>,
{
    fn default() -> Self {
        let mut offsets = Buffer::Buffer::<OffsetItem>::default();
        offsets.extend(iter::once(OffsetItem::default()));
        Self {
            data: T::default(),
            offsets,
        }
    }
}

impl<T: Default, OffsetItem: Offset, Buffer> Default for Offsets<T, Nullable, OffsetItem, Buffer>
where
    Buffer: BufferType<Buffer<OffsetItem>: Extend<OffsetItem>>,
    Validity<<Buffer as BufferType>::Buffer<OffsetItem>, Buffer>: Default,
{
    fn default() -> Self {
        let mut offsets =
            <Nullable as Nullability>::Collection::<Buffer::Buffer<OffsetItem>, Buffer>::default();
        offsets.data.extend(iter::once(OffsetItem::default()));
        Self {
            data: T::default(),
            offsets,
        }
    }
}

impl<T, U, OffsetItem: Offset, Buffer> Extend<U> for Offsets<T, NonNullable, OffsetItem, Buffer>
where
    T: Extend<<U as IntoIterator>::Item>,
    U: IntoIterator + Length,
    Buffer: BufferType<Buffer<OffsetItem>: Extend<OffsetItem>>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        let mut state = self
            .offsets
            .as_slice()
            .last()
            .copied()
            .expect("at least one value in the offsets buffer");
        self.data.extend(
            iter.into_iter()
                .inspect(|item| {
                    state = state
                        .checked_add_unsigned(
                            OffsetItem::Unsigned::try_from(item.len()).expect("len overflow"),
                        )
                        .expect("offset value overflow");
                    self.offsets.extend(iter::once(state));
                })
                .flatten(),
        );
    }
}

impl<T, U, OffsetItem: Offset, Buffer: BufferType> Extend<Option<U>>
    for Offsets<T, Nullable, OffsetItem, Buffer>
where
    T: Extend<<U as IntoIterator>::Item>,
    U: IntoIterator + Length,
    Buffer: BufferType,
    Validity<Buffer::Buffer<OffsetItem>, Buffer>: Extend<(bool, OffsetItem)>,
{
    fn extend<I: IntoIterator<Item = Option<U>>>(&mut self, iter: I) {
        let mut state = self
            .offsets
            .as_ref()
            .as_slice()
            .last()
            .copied()
            .expect("at least one value in the offsets buffer");
        self.data.extend(
            iter.into_iter()
                .inspect(|opt| {
                    state = state
                        .checked_add_unsigned(
                            OffsetItem::Unsigned::try_from(opt.len()).expect("len overflow"),
                        )
                        .expect("offset value overflow");
                    self.offsets.extend(iter::once((opt.is_some(), state)));
                })
                .flatten()
                .flatten(),
        );
    }
}

impl<T, OffsetItem: Offset, Buffer> From<Offsets<T, NonNullable, OffsetItem, Buffer>>
    for Offsets<T, Nullable, OffsetItem, Buffer>
where
    Buffer: BufferType<Buffer<OffsetItem>: Length>,
    Bitmap<Buffer>: FromIterator<bool>,
{
    fn from(value: Offsets<T, NonNullable, OffsetItem, Buffer>) -> Self {
        // Not using `Nullable::wrap` because the offset buffer has one more
        // element than the length.
        let validity = Bitmap::new_valid(value.len());
        Self {
            data: value.data,
            offsets: Validity {
                data: value.offsets,
                validity,
            },
        }
    }
}

impl<T, U, OffsetItem: Offset, Buffer> FromIterator<U>
    for Offsets<T, NonNullable, OffsetItem, Buffer>
where
    Self: Default,
    T: Extend<<U as IntoIterator>::Item>,
    U: IntoIterator + Length,
    Buffer: BufferType<Buffer<OffsetItem>: Extend<OffsetItem>>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        let mut offset = Self::default();
        offset.extend(iter);
        offset
    }
}

impl<T, U, OffsetItem: Offset, Buffer: BufferType> FromIterator<Option<U>>
    for Offsets<T, Nullable, OffsetItem, Buffer>
where
    Self: Default,
    T: Extend<<U as IntoIterator>::Item>,
    U: IntoIterator + Length,
    <Nullable as Nullability>::Collection<Buffer::Buffer<OffsetItem>, Buffer>:
        Extend<(bool, OffsetItem)>,
{
    fn from_iter<I: IntoIterator<Item = Option<U>>>(iter: I) -> Self {
        let mut offset = Self::default();
        offset.extend(iter);
        offset
    }
}

impl<T, U, OffsetItem: Offset, Buffer: BufferType> FromIterator<std::option::IntoIter<U>>
    for Offsets<T, Nullable, OffsetItem, Buffer>
where
    Self: Default,
    T: Extend<<U as IntoIterator>::Item>,
    U: IntoIterator + Length,
    <Nullable as Nullability>::Collection<Buffer::Buffer<OffsetItem>, Buffer>:
        Extend<(bool, OffsetItem)>,
{
    fn from_iter<I: IntoIterator<Item = std::option::IntoIter<U>>>(iter: I) -> Self {
        let mut offset = Self::default();
        offset.extend(iter.into_iter().map(|mut v| v.next()));
        offset
    }
}

/// An iterator over items in an offset.
pub struct OffsetSlice<'a, T, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType> {
    /// The offsets storing the values and offsets
    offset: &'a Offsets<T, Nulls, OffsetItem, Buffer>,
    /// The current position
    index: usize,
    /// Then end of this slice
    end: usize,
}

// TODO(mbrobbel): this is the remaining items in the iterator, maybe we want
// this to be the original slot length?
impl<T, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType> Length
    for OffsetSlice<'_, T, Nulls, OffsetItem, Buffer>
{
    #[inline]
    fn len(&self) -> usize {
        self.end - self.index
    }
}

impl<'a, T, Nulls: Nullability, OffsetItem: Offset, Buffer: BufferType> Iterator
    for OffsetSlice<'a, T, Nulls, OffsetItem, Buffer>
where
    T: Index,
{
    type Item = <T as Index>::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        (self.index < self.end).then(|| {
            // Safety:
            // - Bounds checked above
            let value = unsafe { self.offset.data.index_unchecked(self.index) };
            self.index += 1;
            value
        })
    }
}

impl<T, OffsetItem: Offset, Buffer: BufferType> Index
    for Offsets<T, NonNullable, OffsetItem, Buffer>
{
    type Item<'a>
        = OffsetSlice<'a, T, NonNullable, OffsetItem, Buffer>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        OffsetSlice {
            offset: self,
            index: (
                // Safety:
                // - Unsafe fn
                unsafe { *self.offsets.as_slice().get_unchecked(index) }
            )
            .try_into()
            .expect("offset value out of range"),
            end: (
                // Safety:
                // - Unsafe fn
                unsafe { *self.offsets.as_slice().get_unchecked(index + 1) }
            )
            .try_into()
            .expect("offset value out of range"),
        }
    }
}

impl<T, OffsetItem: Offset, Buffer: BufferType> Index for Offsets<T, Nullable, OffsetItem, Buffer> {
    type Item<'a>
        = Option<OffsetSlice<'a, T, Nullable, OffsetItem, Buffer>>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        // Safety:
        // - TODO
        let start = unsafe {
            (*self.offsets.data.as_slice().get_unchecked(index))
                .try_into()
                .expect("out of bounds")
        };
        // Safety:
        // - TODO
        let end = unsafe {
            (*self.offsets.data.as_slice().get_unchecked(index + 1))
                .try_into()
                .expect("out of bounds")
        };
        // Safety:
        // - TODO
        unsafe {
            self.is_valid_unchecked(index).then_some(OffsetSlice {
                offset: self,
                index: start,
                end,
            })
        }
    }
}

/// An iterator over an offset.
pub struct OffsetIter<'a, Nulls: Nullability, T, OffsetItem: Offset, Buffer: BufferType> {
    /// The offset being iterated over
    offset: &'a Offsets<T, Nulls, OffsetItem, Buffer>,
    /// The current position of this iterator
    position: usize,
}

impl<'a, Nulls: Nullability, T, OffsetItem: Offset, Buffer> Iterator
    for OffsetIter<'a, Nulls, T, OffsetItem, Buffer>
where
    Buffer: BufferType,
    Offsets<T, Nulls, OffsetItem, Buffer>: Index,
{
    type Item = <Offsets<T, Nulls, OffsetItem, Buffer> as Index>::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        (self.position < self.offset.len()).then(|| {
            // Safety:
            // - Bounds checked above
            let value = unsafe { self.offset.index_unchecked(self.position) };
            self.position += 1;
            value
        })
    }
}

impl<'a, Nulls: Nullability, T, OffsetItem: Offset, Buffer> IntoIterator
    for &'a Offsets<T, Nulls, OffsetItem, Buffer>
where
    Buffer: BufferType,
    Offsets<T, Nulls, OffsetItem, Buffer>: Index,
{
    type Item = <Offsets<T, Nulls, OffsetItem, Buffer> as Index>::Item<'a>;
    type IntoIter = OffsetIter<'a, Nulls, T, OffsetItem, Buffer>;

    fn into_iter(self) -> Self::IntoIter {
        OffsetIter {
            offset: self,
            position: 0,
        }
    }
}

/// An owned iterator over an offset
pub struct OffsetIntoIter<T, OffsetItem: Offset, Buffer>
where
    T: IntoIterator,
    Buffer: BufferType<Buffer<OffsetItem>: IntoIterator>,
{
    /// The underlying array data
    data: <T as IntoIterator>::IntoIter,
    /// A peekable offsets buffer
    offsets: Peekable<<<Buffer as BufferType>::Buffer<OffsetItem> as IntoIterator>::IntoIter>,
}

impl<T, OffsetItem: Offset, Buffer> Iterator for OffsetIntoIter<T, OffsetItem, Buffer>
where
    T: IntoIterator,
    Buffer: BufferType<Buffer<OffsetItem>: IntoIterator<Item = OffsetItem>>,
{
    type Item = Vec<<T as IntoIterator>::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.offsets
            .next()
            .and_then(|start: OffsetItem| self.offsets.peek().map(|end: &OffsetItem| (start, *end)))
            .map(|(start, end)| {
                let slot_start: usize = start.try_into().expect("offset value should be in range");
                let slot_end: usize = end.try_into().expect("offset value should be in range");
                let slot_length = slot_end
                    .checked_sub(slot_start)
                    .expect("offsets should be monotonically increasing");

                let taken = self.data.by_ref().take(slot_length).collect::<Vec<_>>();

                debug_assert!(
                    taken.len() == slot_length,
                    "underlying data array does not have enough elements, \
                    expected {} elements for this slot, found {}",
                    slot_length,
                    taken.len()
                );

                taken
            })
    }
}

impl<T, OffsetItem: Offset, Buffer> IntoIterator for Offsets<T, NonNullable, OffsetItem, Buffer>
where
    T: IntoIterator,
    Buffer: BufferType<Buffer<OffsetItem>: IntoIterator<Item = OffsetItem>>,
{
    type Item = Vec<<T as IntoIterator>::Item>;
    type IntoIter = OffsetIntoIter<T, OffsetItem, Buffer>;

    fn into_iter(self) -> Self::IntoIter {
        OffsetIntoIter {
            data: self.data.into_iter(),
            offsets: self.offsets.into_iter().peekable(),
        }
    }
}

impl<T, OffsetItem: Offset, Buffer> IntoIterator for Offsets<T, Nullable, OffsetItem, Buffer>
where
    T: IntoIterator,
    Bitmap<Buffer>: IntoIterator<Item = bool>,
    Buffer: BufferType<Buffer<OffsetItem>: IntoIterator<Item = OffsetItem>>,
{
    type Item = Option<Vec<<T as IntoIterator>::Item>>;
    type IntoIter = Map<
        Zip<
            <Bitmap<Buffer> as IntoIterator>::IntoIter,
            <OffsetIntoIter<T, OffsetItem, Buffer> as IntoIterator>::IntoIter,
        >,
        fn((bool, Vec<<T as IntoIterator>::Item>)) -> Self::Item,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.offsets
            .validity
            .into_iter()
            // We use zip insead of map because a null value may have a
            // positive slot length. That is, a null value may occupy a
            // non-empty memory space in the data buffer.
            // Thus the underlying iterator must be advanced even for null
            // values as indicated by the validity bitmap.
            .zip(OffsetIntoIter {
                data: self.data.into_iter(),
                offsets: self.offsets.data.into_iter().peekable(),
            })
            .map(|(validity, value): (bool, Vec<_>)| validity.then_some(value))
    }
}

impl<T, OffsetItem: Offset, Buffer: BufferType> Length
    for Offsets<T, NonNullable, OffsetItem, Buffer>
{
    fn len(&self) -> usize {
        // The offsets buffer has an additional value
        self.offsets
            .len()
            .checked_sub(1)
            .expect("offset len underflow")
    }
}

impl<T, OffsetItem: Offset, Buffer: BufferType> Length
    for Offsets<T, Nullable, OffsetItem, Buffer>
{
    fn len(&self) -> usize {
        // The offsets contains a bitmap that uses the number of bits as length
        self.offsets.len()
    }
}

impl<T, OffsetItem: Offset, Buffer: BufferType> BitmapRef
    for Offsets<T, Nullable, OffsetItem, Buffer>
{
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.offsets.bitmap_ref()
    }
}

impl<T, OffsetItem: Offset, Buffer: BufferType> BitmapRefMut
    for Offsets<T, Nullable, OffsetItem, Buffer>
{
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.offsets.bitmap_ref_mut()
    }
}

impl<T, OffsetItem: Offset, Buffer: BufferType> ValidityBitmap
    for Offsets<T, Nullable, OffsetItem, Buffer>
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let offset = Offsets::<()>::default();
        assert_eq!(offset.offsets.as_slice(), &[0]);
    }

    #[test]
    fn default_nullable() {
        let offset = Offsets::<(), Nullable>::default();
        assert_eq!(offset.offsets.data.as_slice(), &[0]);
        assert_eq!(offset.len(), 0);
    }

    #[test]
    fn extend() {
        let mut offset = Offsets::<Vec<Vec<u8>>>::default();
        offset.extend(std::iter::once(vec![vec![1, 2, 3, 4], vec![5]]));
        dbg!(&offset.data);
        dbg!(&offset.offsets);
        assert_eq!(offset.len(), 1);
        assert_eq!(offset.offsets.as_slice(), &[0, 2]);
        offset.extend(std::iter::once(vec![vec![5]]));
        assert_eq!(offset.offsets.as_slice(), &[0, 2, 3]);
        assert_eq!(offset.len(), 2);
    }

    #[test]
    fn extend_nullable() {
        let mut offset = Offsets::<Vec<u8>, Nullable>::default();
        offset.extend(vec![Some(vec![1, 2, 3, 4]), None, None]);
        assert_eq!(offset.offsets.as_ref().as_slice(), &[0, 4, 4, 4]);
        assert_eq!(offset.len(), 3);
    }

    #[test]
    fn extend_nullable_string() {
        let mut offset = Offsets::<Vec<u8>, Nullable>::default();
        offset.extend(vec![
            Some("as".to_owned().into_bytes()),
            None,
            Some("df".to_owned().into_bytes()),
        ]);
        assert_eq!(offset.data.as_slice(), "asdf".as_bytes());
        assert_eq!(offset.offsets.bitmap_ref().valid_count(), 2);
        assert_eq!(offset.offsets.bitmap_ref().null_count(), 1);
        assert_eq!(offset.offsets.as_ref().as_slice(), &[0, 2, 2, 4]);
    }

    #[test]
    fn from_iter() {
        let input = vec![vec![1, 2, 3, 4], vec![5, 6], vec![7, 8, 9]];
        let offset = input.into_iter().collect::<Offsets<Vec<u8>>>();
        assert_eq!(offset.len(), 3);
        assert_eq!(offset.offsets.as_slice(), &[0, 4, 6, 9]);
        assert_eq!(offset.data, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn from_iter_nullable() {
        let input = vec![Some(["a".to_owned()]), None, Some(["b".to_owned()]), None];
        let offset = input.into_iter().collect::<Offsets<String, Nullable>>();
        assert_eq!(offset.len(), 4);
        assert_eq!(offset.offsets.as_ref().as_slice(), &[0, 1, 1, 2, 2]);
        assert_eq!(offset.data, "ab");
    }

    #[test]
    fn index() {
        let input = vec![vec![1, 2, 3, 4], vec![5, 6], vec![7, 8, 9]];
        let offset = input.into_iter().collect::<Offsets<Vec<u8>>>();
        let mut first = offset.index_checked(0);
        assert_eq!(first.next(), Some(&1));
        assert_eq!(first.next(), Some(&2));
        assert_eq!(first.next(), Some(&3));
        assert_eq!(first.next(), Some(&4));
        assert_eq!(first.next(), None);

        let input_nullable = vec![Some(vec![1, 2, 3, 4]), None, Some(vec![5, 6, 7, 8])];
        let offset_nullable = input_nullable
            .into_iter()
            .collect::<Offsets<Vec<u8>, Nullable>>();
        let first_opt = offset_nullable.index_checked(0).expect("a value");
        assert_eq!(first_opt.copied().collect::<Vec<_>>(), [1, 2, 3, 4]);
        assert!(offset_nullable.index_checked(1).is_none());
    }

    #[test]
    fn iter() {
        let input = vec![vec![1, 2, 3, 4], vec![5, 6], vec![7, 8, 9]];
        let offset = input.clone().into_iter().collect::<Offsets<Vec<u8>>>();
        let mut iter = offset.iter();
        assert_eq!(iter.next().expect("a value").len(), 4);
        assert_eq!(iter.next().expect("a value").len(), 2);
        assert_eq!(iter.next().expect("a value").len(), 3);
        assert!(iter.next().is_none());
        assert_eq!(
            offset.into_iter().flatten().collect::<Vec<_>>(),
            [1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
    }

    #[test]
    fn into_iter() {
        let input = vec![vec![1, 2, 3, 4], vec![5, 6], vec![7, 8, 9]];
        let offset = input.clone().into_iter().collect::<Offsets<Vec<u8>>>();
        assert_eq!(offset.into_iter().collect::<Vec<_>>(), input);
    }

    #[test]
    fn into_iter_non_empty_null() {
        let input_nullable = vec![Some(vec![1, 2, 3, 4]), None, Some(vec![5, 6, 7, 8])];
        let mut offset_nullable = input_nullable
            .clone()
            .into_iter()
            .collect::<Offsets<Vec<u8>, Nullable>>();

        assert_eq!(
            offset_nullable.offsets.bitmap_ref_mut().is_valid(0),
            Some(true)
        );
        {
            let mut validity = offset_nullable
                .offsets
                .validity
                .iter()
                .collect::<Vec<bool>>();
            if let Some(first) = validity.get_mut(0) {
                // invalidate the first item
                *first = false;
            }
            // replace the validity bitmap
            offset_nullable.offsets.validity = validity.into_iter().collect();
        };
        assert_eq!(
            offset_nullable.offsets.bitmap_ref_mut().is_valid(0),
            Some(false)
        );
        // there's still non-empty data corresponding to the now-invalidated
        // first item in the underlying data array
        assert_eq!(offset_nullable.data.as_slice(), &[1, 2, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            offset_nullable.into_iter().collect::<Vec<_>>(),
            // the first item is not yielded even though there's data for it
            // because the validity bitmap was altered.
            vec![None, None, Some(vec![5, 6, 7, 8])]
        );
    }

    #[test]
    fn convert() {
        let input = vec![vec![1, 2, 3, 4], vec![5, 6], vec![7, 8, 9]];
        let offset = input.into_iter().collect::<Offsets<Vec<u8>>>();
        assert_eq!(offset.len(), 3);
        let offset_nullable: Offsets<Vec<u8>, Nullable> = offset.into();
        assert_eq!(offset_nullable.len(), 3);
        assert!(offset_nullable.all_valid());
    }
}
