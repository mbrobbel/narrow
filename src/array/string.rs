//! Array with string values.

use std::str;

use super::{Array, VariableSizeBinaryArray};
use crate::{
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{BufferType, VecBuffer},
    offset::OffsetElement,
    validity::Validity,
    Index, Length,
};

/// Array with string values.
pub struct StringArray<
    const NULLABLE: bool = false,
    OffsetItem: OffsetElement = i32,
    Buffer: BufferType = VecBuffer,
>(pub VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>)
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>;

/// Array with string values, using `i32` offset values.
pub type Utf8Array<const NULLABLE: bool = false, Buffer = VecBuffer> =
    StringArray<NULLABLE, i32, Buffer>;

/// Array with string values, using `i64` offset values.
pub type LargeUtf8Array<const NULLABLE: bool = false, Buffer = VecBuffer> =
    StringArray<NULLABLE, i64, Buffer>;

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Array
    for StringArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
{
    type Item = String;
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Default
    for StringArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>: Default,
{
    fn default() -> Self {
        Self(VariableSizeBinaryArray::default())
    }
}

impl<'a, T: ?Sized, OffsetItem: OffsetElement, Buffer: BufferType> Extend<&'a T>
    for StringArray<false, OffsetItem, Buffer>
where
    T: AsRef<str>,
    VariableSizeBinaryArray<false, OffsetItem, Buffer>: Extend<&'a [u8]>,
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.0
            .extend(iter.into_iter().map(|item| item.as_ref().as_bytes()));
    }
}

impl<'a, T: ?Sized, OffsetItem: OffsetElement, Buffer: BufferType> Extend<Option<&'a T>>
    for StringArray<true, OffsetItem, Buffer>
where
    T: AsRef<str>,
    VariableSizeBinaryArray<true, OffsetItem, Buffer>: Extend<Option<&'a [u8]>>,
{
    fn extend<I: IntoIterator<Item = Option<&'a T>>>(&mut self, iter: I) {
        self.0.extend(
            iter.into_iter()
                .map(|opt| opt.map(|item| item.as_ref().as_bytes())),
        );
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> Extend<String>
    for StringArray<false, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<false, OffsetItem, Buffer>: Extend<Vec<u8>>,
{
    fn extend<I: IntoIterator<Item = String>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().map(String::into_bytes));
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> Extend<Option<String>>
    for StringArray<true, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<true, OffsetItem, Buffer>: Extend<Option<Vec<u8>>>,
{
    fn extend<I: IntoIterator<Item = Option<String>>>(&mut self, iter: I) {
        self.0
            .extend(iter.into_iter().map(|opt| opt.map(String::into_bytes)));
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> From<StringArray<false, OffsetItem, Buffer>>
    for StringArray<true, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<false, OffsetItem, Buffer>:
        Into<VariableSizeBinaryArray<true, OffsetItem, Buffer>>,
{
    fn from(value: StringArray<false, OffsetItem, Buffer>) -> Self {
        Self(value.0.into())
    }
}

impl<'a, T: ?Sized, OffsetItem: OffsetElement, Buffer: BufferType> FromIterator<&'a T>
    for StringArray<false, OffsetItem, Buffer>
where
    T: AsRef<str>,
    VariableSizeBinaryArray<false, OffsetItem, Buffer>: FromIterator<&'a [u8]>,
{
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|item| item.as_ref().as_bytes())
                .collect(),
        )
    }
}

impl<'a, T: ?Sized, OffsetItem: OffsetElement, Buffer: BufferType> FromIterator<Option<&'a T>>
    for StringArray<true, OffsetItem, Buffer>
where
    T: AsRef<str>,
    VariableSizeBinaryArray<true, OffsetItem, Buffer>: FromIterator<Option<&'a [u8]>>,
{
    fn from_iter<I: IntoIterator<Item = Option<&'a T>>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|x| x.map(|item| item.as_ref().as_bytes()))
                .collect(),
        )
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> FromIterator<String>
    for StringArray<false, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<false, OffsetItem, Buffer>: FromIterator<Vec<u8>>,
{
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self(iter.into_iter().map(String::into_bytes).collect())
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> FromIterator<Option<String>>
    for StringArray<true, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<true, OffsetItem, Buffer>: FromIterator<Option<Vec<u8>>>,
{
    fn from_iter<I: IntoIterator<Item = Option<String>>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|x| x.map(String::into_bytes))
                .collect(),
        )
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> Index
    for StringArray<false, OffsetItem, Buffer>
{
    type Item<'a> = &'a str
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        str::from_utf8_unchecked(self.0.index_unchecked(index))
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> Index
    for StringArray<true, OffsetItem, Buffer>
{
    type Item<'a> = Option<&'a str>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.0
            .index_unchecked(index)
            .map(|bytes| str::from_utf8_unchecked(bytes))
    }
}

/// An iterator over strings in a [`StringArray`].
pub struct StringIter<'a, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
{
    /// Reference to the array.
    array: &'a StringArray<NULLABLE, OffsetItem, Buffer>,
    /// Current index.
    index: usize,
}

impl<'a, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Iterator
    for StringIter<'a, NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    StringArray<NULLABLE, OffsetItem, Buffer>: Length + Index,
{
    type Item = <StringArray<NULLABLE, OffsetItem, Buffer> as Index>::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.array
            .index(self.index)
            .into_iter()
            .inspect(|_| {
                self.index += 1;
            })
            .next()
    }
}

impl<'a, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> IntoIterator
    for &'a StringArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    StringArray<NULLABLE, OffsetItem, Buffer>: Index + Length,
{
    type Item = <StringArray<NULLABLE, OffsetItem, Buffer> as Index>::Item<'a>;
    type IntoIter = StringIter<'a, NULLABLE, OffsetItem, Buffer>;

    fn into_iter(self) -> Self::IntoIter {
        StringIter {
            array: self,
            index: 0,
        }
    }
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Length
    for StringArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> BitmapRef
    for StringArray<true, OffsetItem, Buffer>
{
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> BitmapRefMut
    for StringArray<true, OffsetItem, Buffer>
{
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> ValidityBitmap
    for StringArray<true, OffsetItem, Buffer>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        array::{union, ArrayType},
        bitmap::BitmapRef,
        buffer::BufferRef,
    };

    #[test]
    fn from_iter() {
        let input = ["1", "23", "456", "7890"];
        let array = input
            .into_iter()
            .collect::<<String as ArrayType>::Array<VecBuffer, i64, union::NA>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0 .0.data.0, b"1234567890");

        let input_string = vec!["a".to_owned(), "sd".to_owned(), "f".to_owned()];
        let array_string = input_string.into_iter().collect::<StringArray>();
        assert_eq!(array_string.len(), 3);
        assert_eq!(array_string.0 .0.data.0, &[97, 115, 100, 102]);
        assert_eq!(array_string.0 .0.offsets, &[0, 1, 3, 4]);
    }

    #[test]
    fn from_iter_nullable() {
        let input = vec![Some("a"), None, Some("sd"), Some("f"), None];
        let array = input.into_iter().collect::<StringArray<true>>();
        assert_eq!(array.len(), 5);
        assert_eq!(array.is_valid(0), Some(true));
        assert_eq!(array.is_valid(1), Some(false));
        assert_eq!(array.is_valid(2), Some(true));
        assert_eq!(array.is_valid(3), Some(true));
        assert_eq!(array.is_valid(4), Some(false));
        assert_eq!(array.is_valid(5), None);
        assert_eq!(array.0 .0.data.0, "asdf".as_bytes());
        assert_eq!(array.0 .0.offsets.as_ref(), &[0, 1, 1, 3, 4, 4]);
        assert_eq!(
            array.bitmap_ref().into_iter().collect::<Vec<_>>(),
            &[true, false, true, true, false]
        );
    }

    #[test]
    fn into_iter() {
        let input = ["1", "23", "456", "7890"];
        let array = input.into_iter().collect::<StringArray>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), input);

        let input_nullable = vec![Some("a"), None, Some("sd"), Some("f"), None];
        let array_nullable = input_nullable
            .clone()
            .into_iter()
            .collect::<StringArray<true>>();
        assert_eq!(
            array_nullable.into_iter().collect::<Vec<_>>(),
            input_nullable
        );
    }

    #[test]
    fn convert_nullable() {
        let input = ["hello", " ", "world"];
        let array = input
            .into_iter()
            .map(ToString::to_string)
            .collect::<StringArray>();
        let nullable: StringArray<true> = array.into();
        assert_eq!(nullable.bitmap_ref().buffer_ref(), &[0b0000_0111]);
    }
}
