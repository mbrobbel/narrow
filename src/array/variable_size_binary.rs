//! Variable-size binary elements.

use super::{Array, FixedSizePrimitiveArray, StringArray, VariableSizeListArray};
use crate::{
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{Buffer, BufferType, VecBuffer},
    offset::{Offset, OffsetElement},
    validity::{Nullability, Validity},
    Index, Length,
};

/// Variable-size binary elements.
pub struct VariableSizeBinaryArray<
    const NULLABLE: bool = false,
    OffsetItem: OffsetElement = i32,
    Buffer: BufferType = VecBuffer,
>(pub Offset<FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, OffsetItem, Buffer>)
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>;

/// Variable-size binary elements, using `i32` offset values.
pub type BinaryArray<const NULLABLE: bool = false, Buffer = VecBuffer> =
    VariableSizeBinaryArray<NULLABLE, i32, Buffer>;

/// Variable-size binary elements, using `i64` offset value.
pub type LargeBinaryArray<const NULLABLE: bool = false, Buffer = VecBuffer> =
    VariableSizeBinaryArray<NULLABLE, i64, Buffer>;

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Array
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Vec<u8>: Nullability<NULLABLE>,
{
    type Item = <Vec<u8> as Nullability<NULLABLE>>::Item;
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Default
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Offset<FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, OffsetItem, Buffer>: Default,
{
    fn default() -> Self {
        Self(Offset::default())
    }
}

impl<T, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Extend<T>
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Offset<FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, OffsetItem, Buffer>: Extend<T>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter);
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
        Self(value.0)
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType>
    From<VariableSizeBinaryArray<false, OffsetItem, Buffer>>
    for VariableSizeBinaryArray<true, OffsetItem, Buffer>
where
    Offset<FixedSizePrimitiveArray<u8, false, Buffer>, false, OffsetItem, Buffer>:
        Into<Offset<FixedSizePrimitiveArray<u8, false, Buffer>, true, OffsetItem, Buffer>>,
{
    fn from(value: VariableSizeBinaryArray<false, OffsetItem, Buffer>) -> Self {
        Self(value.0.into())
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
    Offset<FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, OffsetItem, Buffer>:
        FromIterator<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> Index
    for VariableSizeBinaryArray<false, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Index,
{
    type Item<'a> = &'a [u8]
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        let start: usize = self
            .0
            .offsets
            .as_slice()
            .index_unchecked(index)
            .to_owned()
            .try_into()
            .expect("convert fail");
        let end: usize = self
            .0
            .offsets
            .as_slice()
            .index_unchecked(index + 1)
            .to_owned()
            .try_into()
            .expect("convert fail");
        &self.0.data.0.as_slice()[start..end]
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> Index
    for VariableSizeBinaryArray<true, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Index,
{
    type Item<'a> = Option<&'a [u8]>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.0.is_valid_unchecked(index).then(|| {
            let start: usize = self
                .0
                .offsets
                .data
                .as_slice()
                .index_unchecked(index)
                .to_owned()
                .try_into()
                .expect("convert fail");
            let end: usize = self
                .0
                .offsets
                .data
                .as_slice()
                .index_unchecked(index + 1)
                .to_owned()
                .try_into()
                .expect("convert fail");
            &self.0.data.0.as_slice()[start..end]
        })
    }
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Length
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Offset<FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, OffsetItem, Buffer>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> BitmapRef
    for VariableSizeBinaryArray<true, OffsetItem, Buffer>
{
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> BitmapRefMut
    for VariableSizeBinaryArray<true, OffsetItem, Buffer>
{
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<OffsetItem: OffsetElement, Buffer: BufferType> ValidityBitmap
    for VariableSizeBinaryArray<true, OffsetItem, Buffer>
{
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
        let input: [&[u8]; 4] = [&[1], &[2, 3], &[4, 5, 6], &[7, 8, 9, 0]];
        let array = input
            .into_iter()
            .map(<[u8]>::to_vec)
            .collect::<VariableSizeBinaryArray>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0.data.0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);
        assert_eq!(array.0.offsets, &[0, 1, 3, 6, 10]);

        let input_vec = vec![vec![1], vec![], vec![2, 3], vec![4]];
        let array_vec = input_vec.into_iter().collect::<VariableSizeBinaryArray>();
        assert_eq!(array_vec.len(), 4);
        assert_eq!(array_vec.0.data.0, &[1, 2, 3, 4]);
        assert_eq!(array_vec.0.offsets, &[0, 1, 1, 3, 4]);
    }

    #[test]
    fn from_iter_nullable() {
        let input: [Option<&[u8]>; 4] = [Some(&[1]), None, Some(&[4, 5, 6]), Some(&[7, 8, 9, 0])];
        let array = input
            .into_iter()
            .map(|x| x.map(<[u8]>::to_vec))
            .collect::<VariableSizeBinaryArray<true>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0.data.0, &[1, 4, 5, 6, 7, 8, 9, 0]);
        assert_eq!(array.0.offsets.as_ref(), &[0, 1, 1, 4, 8]);
        assert_eq!(array.0.offsets.bitmap_ref().buffer_ref(), &[0b000_01101]);

        let input_vec = vec![Some(vec![1]), None, Some(vec![2, 3]), Some(vec![4])];
        let array_vec = input_vec
            .into_iter()
            .collect::<VariableSizeBinaryArray<true>>();
        assert_eq!(array_vec.len(), 4);
        assert_eq!(array_vec.0.data.0, &[1, 2, 3, 4]);
        assert_eq!(array_vec.0.offsets.as_ref(), &[0, 1, 1, 3, 4]);
        assert_eq!(
            array_vec
                .0
                .offsets
                .bitmap_ref()
                .into_iter()
                .collect::<Vec<_>>(),
            &[true, false, true, true]
        );
    }

    #[test]
    fn index() {
        let input: [&[u8]; 4] = [&[1], &[2, 3], &[4, 5, 6], &[7, 8, 9, 0]];
        let array = input
            .into_iter()
            .map(<[u8]>::to_vec)
            .collect::<VariableSizeBinaryArray>();
        assert_eq!(array.index_checked(0), &[1]);
        assert_eq!(array.index_checked(1), &[2, 3]);
        assert_eq!(array.index_checked(2), &[4, 5, 6]);
        assert_eq!(array.index_checked(3), &[7, 8, 9, 0]);
        assert!(array.index(4).is_none());
    }

    #[test]
    fn convert() {
        let input = vec![Some("a".to_owned()), None, Some("b".to_owned())];
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
