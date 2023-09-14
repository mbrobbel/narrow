//! Offsets for variable-sized arrays.

use crate::{
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{Buffer, BufferType, VecBuffer},
    nullable::Nullable,
    validity::Validity,
    FixedSize, Index, Length,
};
use std::{
    iter,
    num::TryFromIntError,
    ops::{AddAssign, Range, Sub},
};

/// Types representing offset values.
///
/// Values with these types can be used to represent offset values.
///
/// This trait is sealed to prevent downstream implementations.
pub trait OffsetElement:
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

/// Private module for a seal trait.
mod sealed {
    /// Sealed trait to seal [`super::OffsetElement`].
    pub trait Sealed {}
    impl<T> Sealed for T where T: super::OffsetElement {}
}

impl OffsetElement for i32 {
    type Unsigned = u32;
    fn checked_add(self, rhs: Self) -> Option<Self> {
        i32::checked_add(self, rhs)
    }

    fn checked_add_unsigned(self, rhs: Self::Unsigned) -> Option<Self> {
        i32::checked_add_unsigned(self, rhs)
    }
}
impl OffsetElement for i64 {
    type Unsigned = u64;
    fn checked_add(self, rhs: Self) -> Option<Self> {
        i64::checked_add(self, rhs)
    }
    fn checked_add_unsigned(self, rhs: Self::Unsigned) -> Option<Self> {
        i64::checked_add_unsigned(self, rhs)
    }
}

/// A reference to a slot in an offset
pub struct OffsetSlot<'a, OffsetItem: OffsetElement, Buffer: BufferType> {
    /// The offset buffer
    offset: &'a <Buffer as BufferType>::Buffer<OffsetItem>,
    /// The position in the offset
    index: usize,
}

impl<'a, OffsetItem: OffsetElement, Buffer: BufferType> OffsetSlot<'a, OffsetItem, Buffer> {
    fn position(&self) -> usize {
        self.index
    }
    fn start(&self) -> OffsetItem {
        unsafe {
            self.offset
                .as_slice()
                .index_unchecked(self.index)
                .to_owned()
        }
    }
    fn start_usize(&self) -> usize {
        self.start().try_into().expect("convert fail")
    }
    fn range(&self) -> Range<OffsetItem> {
        self.start()..self.end()
    }
    fn range_usize(&self) -> Range<usize> {
        self.start_usize()..self.end_usize()
    }
    fn end(&self) -> OffsetItem {
        unsafe {
            self.offset
                .as_slice()
                .index_unchecked(self.index + 1)
                .to_owned()
        }
    }
    fn end_usize(&self) -> usize {
        self.end().try_into().expect("convert fail")
    }
    fn len(&self) -> OffsetItem {
        self.end() - self.start()
    }
    fn len_usize(&self) -> usize {
        self.len().try_into().expect("convert fail")
    }
    fn tuple(&self) -> (OffsetItem, OffsetItem) {
        (self.start(), self.end())
    }
    fn tuple_usize(&self) -> (usize, usize) {
        (self.start_usize(), self.end_usize())
    }
}

/// Offset abstraction.
pub struct Offset<
    T,
    const NULLABLE: bool = false,
    OffsetItem: OffsetElement = i32,
    Buffer: BufferType = VecBuffer,
> where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
{
    /// The data
    pub data: T,
    /// The offsets
    pub offsets:
        <<Buffer as BufferType>::Buffer<OffsetItem> as Validity<NULLABLE>>::Storage<Buffer>,
}

impl<T: Default, OffsetItem: OffsetElement, Buffer: BufferType> Default
    for Offset<T, false, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Default + Extend<OffsetItem>,
{
    fn default() -> Self {
        let mut offsets = <Buffer as BufferType>::Buffer::<OffsetItem>::default();
        offsets.extend(iter::once(OffsetItem::default()));
        Self {
            data: T::default(),
            offsets,
        }
    }
}

impl<T: Default, OffsetItem: OffsetElement, Buffer: BufferType> Default
    for Offset<T, true, OffsetItem, Buffer>
where
    <<Buffer as BufferType>::Buffer<OffsetItem> as Validity<true>>::Storage<Buffer>: Default,
    <Buffer as BufferType>::Buffer<OffsetItem>: Extend<OffsetItem>,
{
    fn default() -> Self {
        let mut offsets = <<Buffer as BufferType>::Buffer<OffsetItem> as Validity<true>>::Storage::<
            Buffer,
        >::default();
        offsets.data.extend(iter::once(OffsetItem::default()));
        Self {
            data: T::default(),
            offsets,
        }
    }
}

impl<T, U: IntoIterator + Length, OffsetItem: OffsetElement, Buffer: BufferType> Extend<U>
    for Offset<T, false, OffsetItem, Buffer>
where
    T: Extend<<U as IntoIterator>::Item>,
    <Buffer as BufferType>::Buffer<OffsetItem>: Extend<OffsetItem>,
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

impl<T, U: IntoIterator + Length, OffsetItem: OffsetElement, Buffer: BufferType> Extend<Option<U>>
    for Offset<T, true, OffsetItem, Buffer>
where
    T: Extend<<U as IntoIterator>::Item>,
    <<Buffer as BufferType>::Buffer<OffsetItem> as Validity<true>>::Storage<Buffer>:
        Extend<(bool, OffsetItem)>,
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

impl<T, OffsetItem: OffsetElement, Buffer: BufferType> From<Offset<T, false, OffsetItem, Buffer>>
    for Offset<T, true, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Length,
    Bitmap<Buffer>: FromIterator<bool>,
{
    fn from(value: Offset<T, false, OffsetItem, Buffer>) -> Self {
        // Not using `Nullable::wrap` because the offset buffer has one more
        // element than the length.
        let validity = Bitmap::new_valid(value.len());
        Self {
            data: value.data,
            offsets: Nullable {
                data: value.offsets,
                validity,
            },
        }
    }
}

impl<T, U: IntoIterator + Length, OffsetItem: OffsetElement, Buffer: BufferType> FromIterator<U>
    for Offset<T, false, OffsetItem, Buffer>
where
    Self: Default,
    T: Extend<<U as IntoIterator>::Item>,
    <Buffer as BufferType>::Buffer<OffsetItem>: Extend<OffsetItem>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        let mut offset = Self::default();
        offset.extend(iter);
        offset
    }
}

impl<T, U: IntoIterator + Length, OffsetItem: OffsetElement, Buffer: BufferType>
    FromIterator<Option<U>> for Offset<T, true, OffsetItem, Buffer>
where
    Self: Default,
    T: Extend<<U as IntoIterator>::Item>,
    <<Buffer as BufferType>::Buffer<OffsetItem> as Validity<true>>::Storage<Buffer>:
        Extend<(bool, OffsetItem)>,
{
    fn from_iter<I: IntoIterator<Item = Option<U>>>(iter: I) -> Self {
        let mut offset = Self::default();
        offset.extend(iter);
        offset
    }
}

/// An iterator over items in an offset.
pub struct OffsetSlice<'a, T, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
{
    /// The offset storing the values and offsets
    offset: &'a Offset<T, NULLABLE, OffsetItem, Buffer>,
    /// The current position
    index: usize,
    /// Then end of this slice
    end: usize,
}

impl<'a, T, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Iterator
    for OffsetSlice<'a, T, NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
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

impl<T, OffsetItem: OffsetElement, Buffer: BufferType> Index
    for Offset<T, false, OffsetItem, Buffer>
{
    type Item<'a> = OffsetSlice<'a, T, false, OffsetItem, Buffer>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        OffsetSlice {
            offset: self,
            index: (*self.offsets.as_slice().get_unchecked(index))
                .try_into()
                .expect("offset value out of range"),
            end: (*self.offsets.as_slice().get_unchecked(index + 1))
                .try_into()
                .expect("offset value out of range"),
        }
    }
}

impl<T, OffsetItem: OffsetElement, Buffer: BufferType> Index
    for Offset<T, true, OffsetItem, Buffer>
{
    type Item<'a> = Option<OffsetSlice<'a, T, true, OffsetItem, Buffer>>
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

impl<T, OffsetItem: OffsetElement, Buffer: BufferType> Length
    for Offset<T, false, OffsetItem, Buffer>
{
    fn len(&self) -> usize {
        // The offsets buffer has an additional value
        self.offsets
            .len()
            .checked_sub(1)
            .expect("offset len underflow")
    }
}

impl<T, OffsetItem: OffsetElement, Buffer: BufferType> Length
    for Offset<T, true, OffsetItem, Buffer>
{
    fn len(&self) -> usize {
        // The offsets contains a bitmap that uses the number of bits as length
        self.offsets.len()
    }
}

impl<T, OffsetItem: OffsetElement, Buffer: BufferType> BitmapRef
    for Offset<T, true, OffsetItem, Buffer>
{
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.offsets.bitmap_ref()
    }
}

impl<T, OffsetItem: OffsetElement, Buffer: BufferType> BitmapRefMut
    for Offset<T, true, OffsetItem, Buffer>
{
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.offsets.bitmap_ref_mut()
    }
}

impl<T, OffsetItem: OffsetElement, Buffer: BufferType> ValidityBitmap
    for Offset<T, true, OffsetItem, Buffer>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitmap::{BitmapRef, ValidityBitmap};

    #[test]
    fn default() {
        let offset = Offset::<()>::default();
        assert_eq!(offset.offsets.as_slice(), &[0]);
    }

    #[test]
    fn default_nullable() {
        let offset = Offset::<(), true>::default();
        assert_eq!(offset.offsets.data.as_slice(), &[0]);
        assert_eq!(offset.len(), 0);
    }

    #[test]
    fn extend() {
        let mut offset = Offset::<Vec<Vec<u8>>>::default();
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
        let mut offset = Offset::<Vec<u8>, true>::default();
        offset.extend(vec![Some(vec![1, 2, 3, 4]), None, None]);
        assert_eq!(offset.offsets.as_ref().as_slice(), &[0, 4, 4, 4]);
        assert_eq!(offset.len(), 3);
    }

    #[test]
    fn extend_nullable_string() {
        let mut offset = Offset::<Vec<u8>, true>::default();
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
        let offset = input.into_iter().collect::<Offset<Vec<u8>>>();
        assert_eq!(offset.len(), 3);
        assert_eq!(offset.offsets.as_slice(), &[0, 4, 6, 9]);
        assert_eq!(offset.data, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn from_iter_nullable() {
        let input = vec![Some(["a".to_owned()]), None, Some(["b".to_owned()]), None];
        let offset = input.into_iter().collect::<Offset<String, true>>();
        assert_eq!(offset.len(), 4);
        assert_eq!(offset.offsets.as_ref().as_slice(), &[0, 1, 1, 2, 2]);
        assert_eq!(offset.data, "ab");
    }

    #[test]
    fn index() {
        let input = vec![vec![1, 2, 3, 4], vec![5, 6], vec![7, 8, 9]];
        let offset = input.into_iter().collect::<Offset<Vec<u8>>>();
        let mut first = offset.index_checked(0);
        assert_eq!(first.next(), Some(&1));
        assert_eq!(first.next(), Some(&2));
        assert_eq!(first.next(), Some(&3));
        assert_eq!(first.next(), Some(&4));
        assert_eq!(first.next(), None);

        let input_nullable = vec![Some(vec![1, 2, 3, 4]), None, Some(vec![5, 6, 7, 8])];
        let offset_nullable = input_nullable
            .into_iter()
            .collect::<Offset<Vec<u8>, true>>();
        let first_opt = offset_nullable.index_checked(0).expect("a value");
        assert_eq!(first_opt.copied().collect::<Vec<_>>(), [1, 2, 3, 4]);
        assert!(offset_nullable.index_checked(1).is_none());
    }

    #[test]
    fn convert() {
        let input = vec![vec![1, 2, 3, 4], vec![5, 6], vec![7, 8, 9]];
        let offset = input.into_iter().collect::<Offset<Vec<u8>>>();
        assert_eq!(offset.len(), 3);
        let offset_nullable: Offset<Vec<u8>, true> = offset.into();
        assert_eq!(offset_nullable.len(), 3);
        assert!(offset_nullable.all_valid());
    }
}
