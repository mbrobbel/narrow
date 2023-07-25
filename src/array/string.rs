use super::{Array, VariableSizeBinaryArray};
use crate::{
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

pub type Utf8Array<const NULLABLE: bool = false, Buffer = VecBuffer> =
    StringArray<NULLABLE, i32, Buffer>;

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
        Self(Default::default())
    }
}

impl<'a, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Extend<&'a str>
    for StringArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>: Extend<&'a [u8]>,
{
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().map(str::as_bytes))
    }
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Extend<String>
    for StringArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>: Extend<Vec<u8>>,
{
    fn extend<I: IntoIterator<Item = String>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().map(String::into_bytes))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitmap::BitmapRef;

    #[test]
    fn from_iter() {
        let input = ["1", "23", "456", "7890"];
        let array = input
            .into_iter()
            .map(ToOwned::to_owned)
            .collect::<Utf8Array>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0 .0 .0.data.0, b"1234567890");

        let input = vec!["a".to_string(), "sd".to_string(), "f".to_string()];
        let array = input.into_iter().collect::<StringArray>();
        assert_eq!(array.len(), 3);
        assert_eq!(array.0 .0 .0.data.0, &[97, 115, 100, 102]);
        assert_eq!(array.0 .0 .0.offsets, &[0, 1, 3, 4]);

        let input = vec![
            Some("a".to_string()),
            None,
            Some("sd".to_string()),
            Some("f".to_string()),
            None,
        ];
        let array = input.into_iter().collect::<StringArray<true>>();
        assert_eq!(array.len(), 5);
        assert_eq!(array.0 .0 .0.data.0, &[97, 115, 100, 102]);
        assert_eq!(array.0 .0 .0.offsets.as_ref(), &[0, 1, 1, 3, 4, 4]);
        assert_eq!(
            array
                .0
                 .0
                 .0
                .offsets
                .bitmap_ref()
                .into_iter()
                .collect::<Vec<_>>(),
            &[true, false, true, true, false]
        );
    }
}
