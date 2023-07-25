//! Variable-size binary elements.

use super::{Array, FixedSizePrimitiveArray, StringArray, VariableSizeListArray};
use crate::{
    buffer::{BufferType, VecBuffer},
    offset::OffsetElement,
    validity::Validity,
    Length,
};

/// Variable-size binary elements.
pub struct VariableSizeBinaryArray<
    const NULLABLE: bool = false,
    OffsetItem: OffsetElement = i32,
    Buffer: BufferType = VecBuffer,
>(
    pub  VariableSizeListArray<
        FixedSizePrimitiveArray<u8, false, Buffer>,
        NULLABLE,
        OffsetItem,
        Buffer,
    >,
)
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>;

pub type BinaryArray<const NULLABLE: bool = false, Buffer = VecBuffer> =
    VariableSizeBinaryArray<NULLABLE, i32, Buffer>;

pub type LargeBinaryArray<const NULLABLE: bool = false, Buffer = VecBuffer> =
    VariableSizeBinaryArray<NULLABLE, i64, Buffer>;

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Array
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
{
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Default
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    VariableSizeListArray<FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, OffsetItem, Buffer>:
        Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Extend<T>
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    VariableSizeListArray<FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, OffsetItem, Buffer>:
        Extend<T>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType>
    From<
        VariableSizeListArray<
            FixedSizePrimitiveArray<u8, false, Buffer>,
            NULLABLE,
            OffsetItem,
            Buffer,
        >,
    > for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
{
    fn from(
        value: VariableSizeListArray<
            FixedSizePrimitiveArray<u8, false, Buffer>,
            NULLABLE,
            OffsetItem,
            Buffer,
        >,
    ) -> Self {
        Self(value)
    }
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType>
    From<StringArray<NULLABLE, OffsetItem, Buffer>>
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
{
    fn from(value: StringArray<NULLABLE, OffsetItem, Buffer>) -> Self {
        Self(value.0 .0)
    }
}

impl<T, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> FromIterator<T>
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    VariableSizeListArray<FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, OffsetItem, Buffer>:
        FromIterator<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Length
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    VariableSizeListArray<FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, OffsetItem, Buffer>:
        Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bitmap::{Bitmap, BitmapRef},
        buffer::BufferRef,
    };
    use std::mem;

    #[test]
    fn from_iter() {
        let input: [&[u8]; 4] = [&[1u8], &[2, 3], &[4, 5, 6], &[7, 8, 9, 0]];
        let array = input
            .into_iter()
            .map(|x| x.to_vec())
            .collect::<VariableSizeBinaryArray>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0 .0.data.0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);
        assert_eq!(array.0 .0.offsets, &[0, 1, 3, 6, 10]);

        let input: [Option<&[u8]>; 4] = [Some(&[1u8]), None, Some(&[4, 5, 6]), Some(&[7, 8, 9, 0])];
        let array = input
            .into_iter()
            .map(|x| x.map(|x| x.to_vec()))
            .collect::<VariableSizeBinaryArray<true>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0 .0.data.0, &[1, 4, 5, 6, 7, 8, 9, 0]);
        assert_eq!(array.0 .0.offsets.as_ref(), &[0, 1, 1, 4, 8]);
        assert_eq!(array.0 .0.offsets.bitmap_ref().buffer_ref(), &[0b00001101]);

        let input = vec![vec![1], vec![], vec![2, 3], vec![4]];
        let array = input.into_iter().collect::<VariableSizeBinaryArray>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0 .0.data.0, &[1, 2, 3, 4]);
        assert_eq!(array.0 .0.offsets, &[0, 1, 1, 3, 4]);

        let input = vec![Some(vec![1]), None, Some(vec![2, 3]), Some(vec![4])];
        let array = input.into_iter().collect::<VariableSizeBinaryArray<true>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0 .0.data.0, &[1, 2, 3, 4]);
        assert_eq!(array.0 .0.offsets.as_ref(), &[0, 1, 1, 3, 4]);
        assert_eq!(
            array
                .0
                 .0
                .offsets
                .bitmap_ref()
                .into_iter()
                .collect::<Vec<_>>(),
            &[true, false, true, true]
        );
    }

    #[test]
    fn convert() {
        let input = vec![Some("a".to_string()), None, Some("b".to_string())];
        let array = input.into_iter().collect::<StringArray<true>>();
        let variable_size_binary: VariableSizeBinaryArray<true> = array.into();
        assert_eq!(variable_size_binary.len(), 3);
    }

    #[test]
    fn size_of() {
        assert_eq!(
            mem::size_of::<BinaryArray>(),
            mem::size_of::<Vec<u8>>() + mem::size_of::<Vec<i32>>()
        );
        assert_eq!(
            mem::size_of::<LargeBinaryArray>(),
            mem::size_of::<Vec<u8>>() + mem::size_of::<Vec<i64>>()
        );
        assert_eq!(
            mem::size_of::<BinaryArray<true>>(),
            mem::size_of::<Vec<u8>>() + mem::size_of::<Vec<i32>>() + mem::size_of::<Bitmap>()
        );
        assert_eq!(
            mem::size_of::<LargeBinaryArray<true>>(),
            mem::size_of::<Vec<u8>>() + mem::size_of::<Vec<i64>>() + mem::size_of::<Bitmap>()
        );
    }
}
