//! Array with string values.

use super::{Array, VariableSizeBinaryArray};
use crate::{
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{BufferType, VecBuffer},
    offset::OffsetElement,
    validity::Validity,
    Length,
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

impl<'a, OffsetItem: OffsetElement, Buffer: BufferType> Extend<&'a str>
    for StringArray<false, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<false, OffsetItem, Buffer>: Extend<&'a [u8]>,
{
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().map(str::as_bytes));
    }
}

impl<'a, OffsetItem: OffsetElement, Buffer: BufferType> Extend<Option<&'a str>>
    for StringArray<true, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<true, OffsetItem, Buffer>: Extend<Option<&'a [u8]>>,
{
    fn extend<I: IntoIterator<Item = Option<&'a str>>>(&mut self, iter: I) {
        self.0
            .extend(iter.into_iter().map(|opt| opt.map(str::as_bytes)));
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

impl<'a, OffsetItem: OffsetElement, Buffer: BufferType> FromIterator<&'a str>
    for StringArray<false, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<false, OffsetItem, Buffer>: FromIterator<&'a [u8]>,
{
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        Self(iter.into_iter().map(str::as_bytes).collect())
    }
}

impl<'a, OffsetItem: OffsetElement, Buffer: BufferType> FromIterator<Option<&'a str>>
    for StringArray<true, OffsetItem, Buffer>
where
    VariableSizeBinaryArray<true, OffsetItem, Buffer>: FromIterator<Option<&'a [u8]>>,
{
    fn from_iter<I: IntoIterator<Item = Option<&'a str>>>(iter: I) -> Self {
        Self(iter.into_iter().map(|x| x.map(str::as_bytes)).collect())
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
    use crate::{bitmap::BitmapRef, buffer::BufferRef};

    #[test]
    fn from_iter() {
        let input = ["1", "23", "456", "7890"];
        let array = input
            .into_iter()
            .map(ToOwned::to_owned)
            .collect::<Utf8Array>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0 .0 .0.data.0, b"1234567890");

        let input_string = vec!["a".to_owned(), "sd".to_owned(), "f".to_owned()];
        let array_string = input_string.into_iter().collect::<StringArray>();
        assert_eq!(array_string.len(), 3);
        assert_eq!(array_string.0 .0 .0.data.0, &[97, 115, 100, 102]);
        assert_eq!(array_string.0 .0 .0.offsets, &[0, 1, 3, 4]);
    }

    #[test]
    fn from_iter_nullable() {
        let input = vec![
            Some("a".to_owned()),
            None,
            Some("sd".to_owned()),
            Some("f".to_owned()),
            None,
        ];
        let array = input.into_iter().collect::<StringArray<true>>();
        assert_eq!(array.len(), 5);
        assert_eq!(array.is_valid(0), Some(true));
        assert_eq!(array.is_valid(1), Some(false));
        assert_eq!(array.is_valid(2), Some(true));
        assert_eq!(array.is_valid(3), Some(true));
        assert_eq!(array.is_valid(4), Some(false));
        assert_eq!(array.is_valid(5), None);
        assert_eq!(array.0 .0 .0.data.0, "asdf".as_bytes());
        assert_eq!(array.0 .0 .0.offsets.as_ref(), &[0, 1, 1, 3, 4, 4]);
        assert_eq!(
            array.bitmap_ref().into_iter().collect::<Vec<_>>(),
            &[true, false, true, true, false]
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
