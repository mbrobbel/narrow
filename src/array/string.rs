//! Array with string values.

use std::{iter::Map, str};

use super::{Array, VariableSizeBinaryArray};
use crate::{
    Index, Length,
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{BufferType, VecBuffer},
    nullability::{NonNullable, Nullability, Nullable},
    offset::Offset,
};

/// Array with string values.
pub struct StringArray<
    Nullable: Nullability = NonNullable,
    OffsetItem: Offset = i32,
    Buffer: BufferType = VecBuffer,
>(pub VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>);

/// Array with string values, using `i32` offset values.
pub type Utf8Array<Nullable = NonNullable, Buffer = VecBuffer> = StringArray<Nullable, i32, Buffer>;

/// Array with string values, using `i64` offset values.
pub type LargeUtf8Array<Nullable = NonNullable, Buffer = VecBuffer> =
    StringArray<Nullable, i64, Buffer>;

impl<Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType>
    StringArray<Nullable, OffsetItem, Buffer>
where
    StringArray<Nullable, OffsetItem, Buffer>: Index + Length,
{
    /// Returns an iterator over the items in this [`StringArray`].
    pub fn iter(&self) -> StringIter<'_, Nullable, OffsetItem, Buffer> {
        <&Self as IntoIterator>::into_iter(self)
    }
}

impl<Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Array
    for StringArray<Nullable, OffsetItem, Buffer>
{
    type Item = Nullable::Item<String>;
}

impl<Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Clone
    for StringArray<Nullable, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Default
    for StringArray<Nullable, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>: Default,
{
    fn default() -> Self {
        Self(VariableSizeBinaryArray::default())
    }
}

impl<'a, T: ?Sized, OffsetItem: Offset, Buffer: BufferType> Extend<&'a T>
    for StringArray<NonNullable, OffsetItem, Buffer>
where
    T: AsRef<str>,
    VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer>: Extend<&'a [u8]>,
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.0
            .extend(iter.into_iter().map(|item| item.as_ref().as_bytes()));
    }
}

impl<'a, T: ?Sized, OffsetItem: Offset, Buffer: BufferType> Extend<Option<&'a T>>
    for StringArray<Nullable, OffsetItem, Buffer>
where
    T: AsRef<str>,
    VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>: Extend<Option<&'a [u8]>>,
{
    fn extend<I: IntoIterator<Item = Option<&'a T>>>(&mut self, iter: I) {
        self.0.extend(
            iter.into_iter()
                .map(|opt| opt.map(|item| item.as_ref().as_bytes())),
        );
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> Extend<String>
    for StringArray<NonNullable, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer>: Extend<Vec<u8>>,
{
    fn extend<I: IntoIterator<Item = String>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().map(String::into_bytes));
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> Extend<Option<String>>
    for StringArray<Nullable, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>: Extend<Option<Vec<u8>>>,
{
    fn extend<I: IntoIterator<Item = Option<String>>>(&mut self, iter: I) {
        self.0
            .extend(iter.into_iter().map(|opt| opt.map(String::into_bytes)));
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> From<StringArray<NonNullable, OffsetItem, Buffer>>
    for StringArray<Nullable, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer>:
        Into<VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>>,
{
    fn from(value: StringArray<NonNullable, OffsetItem, Buffer>) -> Self {
        Self(value.0.into())
    }
}

impl<'a, T: ?Sized, OffsetItem: Offset, Buffer: BufferType> FromIterator<&'a T>
    for StringArray<NonNullable, OffsetItem, Buffer>
where
    T: AsRef<str>,
    VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer>: FromIterator<&'a [u8]>,
{
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|item| item.as_ref().as_bytes())
                .collect(),
        )
    }
}

impl<'a, T: ?Sized, OffsetItem: Offset, Buffer: BufferType> FromIterator<Option<&'a T>>
    for StringArray<Nullable, OffsetItem, Buffer>
where
    T: AsRef<str>,
    VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>: FromIterator<Option<&'a [u8]>>,
{
    fn from_iter<I: IntoIterator<Item = Option<&'a T>>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|x| x.map(|item| item.as_ref().as_bytes()))
                .collect(),
        )
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> FromIterator<String>
    for StringArray<NonNullable, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer>: FromIterator<Vec<u8>>,
{
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self(iter.into_iter().map(String::into_bytes).collect())
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> FromIterator<Option<String>>
    for StringArray<Nullable, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>: FromIterator<Option<Vec<u8>>>,
{
    fn from_iter<I: IntoIterator<Item = Option<String>>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|x| x.map(String::into_bytes))
                .collect(),
        )
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> Index
    for StringArray<NonNullable, OffsetItem, Buffer>
{
    type Item<'a>
        = &'a str
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        str::from_utf8_unchecked(self.0.index_unchecked(index))
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> Index for StringArray<Nullable, OffsetItem, Buffer> {
    type Item<'a>
        = Option<&'a str>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.0
            .index_unchecked(index)
            .map(|bytes| str::from_utf8_unchecked(bytes))
    }
}

/// An iterator over strings in a [`StringArray`].
pub struct StringIter<'a, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> {
    /// Reference to the array.
    array: &'a StringArray<Nullable, OffsetItem, Buffer>,
    /// Current index.
    index: usize,
}

impl<'a, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Iterator
    for StringIter<'a, Nullable, OffsetItem, Buffer>
where
    StringArray<Nullable, OffsetItem, Buffer>: Length + Index,
{
    type Item = <StringArray<Nullable, OffsetItem, Buffer> as Index>::Item<'a>;

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

impl<'a, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> IntoIterator
    for &'a StringArray<Nullable, OffsetItem, Buffer>
where
    StringArray<Nullable, OffsetItem, Buffer>: Index + Length,
{
    type Item = <StringArray<Nullable, OffsetItem, Buffer> as Index>::Item<'a>;
    type IntoIter = StringIter<'a, Nullable, OffsetItem, Buffer>;

    fn into_iter(self) -> Self::IntoIter {
        StringIter {
            array: self,
            index: 0,
        }
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> IntoIterator
    for StringArray<NonNullable, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer>: IntoIterator<Item = Vec<u8>>,
{
    type Item = String;
    type IntoIter = Map<
        <VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer> as IntoIterator>::IntoIter,
        fn(
            <VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer> as IntoIterator>::Item,
        ) -> String,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(|bytes| {
            // SAFETY:
            // - String arrays contain valid UTF8.
            unsafe { String::from_utf8_unchecked(bytes) }
        })
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> IntoIterator
    for StringArray<Nullable, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>: IntoIterator<Item = Option<Vec<u8>>>,
{
    type Item = Option<String>;
    type IntoIter = Map<
        <VariableSizeBinaryArray<Nullable, OffsetItem, Buffer> as IntoIterator>::IntoIter,
        fn(
            <VariableSizeBinaryArray<Nullable, OffsetItem, Buffer> as IntoIterator>::Item,
        ) -> Option<String>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(|opt| {
            opt.map(|bytes| {
                // SAFETY:
                // - String arrays contain valid UTF8.
                unsafe { String::from_utf8_unchecked(bytes) }
            })
        })
    }
}

impl<Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Length
    for StringArray<Nullable, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> BitmapRef
    for StringArray<Nullable, OffsetItem, Buffer>
{
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> BitmapRefMut
    for StringArray<Nullable, OffsetItem, Buffer>
{
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> ValidityBitmap
    for StringArray<Nullable, OffsetItem, Buffer>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{array::ArrayTypeOf, buffer::BufferRef};

    #[test]
    fn from_iter() {
        let input = ["1", "23", "456", "7890"];
        let array = input.into_iter().collect::<ArrayTypeOf<String>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0.0.data.0, b"1234567890");

        let input_string = vec!["a".to_owned(), "sd".to_owned(), "f".to_owned()];
        let array_string = input_string.into_iter().collect::<StringArray>();
        assert_eq!(array_string.len(), 3);
        assert_eq!(array_string.0.0.data.0, &[97, 115, 100, 102]);
        assert_eq!(array_string.0.0.offsets, &[0, 1, 3, 4]);
    }

    #[test]
    fn from_iter_nullable() {
        let input = vec![Some("a"), None, Some("sd"), Some("f"), None];
        let array = input.into_iter().collect::<StringArray<Nullable>>();
        assert_eq!(array.len(), 5);
        assert_eq!(array.is_valid(0), Some(true));
        assert_eq!(array.is_valid(1), Some(false));
        assert_eq!(array.is_valid(2), Some(true));
        assert_eq!(array.is_valid(3), Some(true));
        assert_eq!(array.is_valid(4), Some(false));
        assert_eq!(array.is_valid(5), None);
        assert_eq!(array.0.0.data.0, "asdf".as_bytes());
        assert_eq!(array.0.0.offsets.as_ref(), &[0, 1, 1, 3, 4, 4]);
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

        let input_nullable = vec![
            Some("a".to_owned()),
            None,
            Some("sd".to_owned()),
            Some("f".to_owned()),
            None,
        ];
        let array_nullable = input_nullable
            .clone()
            .into_iter()
            .collect::<StringArray<Nullable>>();
        let output = array_nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(output, input_nullable);
    }

    #[test]
    fn convert_nullable() {
        let input = ["hello", " ", "world"];
        let array = input
            .into_iter()
            .map(ToOwned::to_owned)
            .collect::<StringArray>();
        let nullable: StringArray<Nullable> = array.into();
        assert_eq!(nullable.bitmap_ref().buffer_ref(), &[0b0000_0111]);
    }
}
