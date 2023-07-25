use super::{Array, FixedSizePrimitiveArray, VariableSizeBinaryArray};
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
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>;

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Array
    for StringArray<NULLABLE, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
{
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Default
    for StringArray<NULLABLE, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<'a, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Extend<&'a str>
    for StringArray<NULLABLE, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>: Extend<&'a [u8]>,
{
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().map(str::as_bytes))
    }
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Extend<String>
    for StringArray<NULLABLE, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
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
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
