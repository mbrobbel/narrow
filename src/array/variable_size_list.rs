//!Array with variable-size list elements.

use crate::{
    array::Array,
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{BufferType, VecBuffer},
    offset::{Offset, OffsetElement},
    validity::Validity,
    Length,
};
use std::fmt::{Debug, Formatter, Result};

/// Array with variable-size list elements.
pub struct VariableSizeListArray<
    T: Array,
    const NULLABLE: bool = false,
    OffsetItem: OffsetElement = i32,
    Buffer: BufferType = VecBuffer,
>(pub Offset<T, NULLABLE, OffsetItem, Buffer>)
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>;

impl<T: Array, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Array
    for VariableSizeListArray<T, NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
{
}

impl<T: Array, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Debug
    for VariableSizeListArray<T, NULLABLE, OffsetItem, Buffer>
where
    Offset<T, NULLABLE, OffsetItem, Buffer>: Debug,
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_tuple("VariableSizeListArray")
            .field(&self.0)
            .finish()
    }
}

impl<T: Array, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Default
    for VariableSizeListArray<T, NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Offset<T, NULLABLE, OffsetItem, Buffer>: Default,
{
    fn default() -> Self {
        Self(Offset::default())
    }
}

impl<T: Array, U, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Extend<U>
    for VariableSizeListArray<T, NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Offset<T, NULLABLE, OffsetItem, Buffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: Array, OffsetItem: OffsetElement, Buffer: BufferType>
    From<VariableSizeListArray<T, false, OffsetItem, Buffer>>
    for VariableSizeListArray<T, true, OffsetItem, Buffer>
where
    Offset<T, false, OffsetItem, Buffer>: Into<Offset<T, true, OffsetItem, Buffer>>,
{
    fn from(value: VariableSizeListArray<T, false, OffsetItem, Buffer>) -> Self {
        Self(value.0.into())
    }
}

impl<T: Array, U, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType>
    FromIterator<U> for VariableSizeListArray<T, NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Offset<T, NULLABLE, OffsetItem, Buffer>: FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: Array, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Length
    for VariableSizeListArray<T, NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Offset<T, NULLABLE, OffsetItem, Buffer>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Array, OffsetItem: OffsetElement, Buffer: BufferType> BitmapRef
    for VariableSizeListArray<T, true, OffsetItem, Buffer>
{
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<T: Array, OffsetItem: OffsetElement, Buffer: BufferType> BitmapRefMut
    for VariableSizeListArray<T, true, OffsetItem, Buffer>
{
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<T: Array, OffsetItem: OffsetElement, Buffer: BufferType> ValidityBitmap
    for VariableSizeListArray<T, true, OffsetItem, Buffer>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        array::FixedSizePrimitiveArray,
        bitmap::{BitmapRef, ValidityBitmap},
    };

    #[test]
    fn from_iter() {
        let input = vec![vec![1], vec![2, 3], vec![4]];
        let array = input
            .into_iter()
            .collect::<VariableSizeListArray<FixedSizePrimitiveArray<u8>>>();
        assert_eq!(array.0.data.0, &[1, 2, 3, 4]);
        assert_eq!(array.0.offsets, &[0, 1, 3, 4]);
    }

    #[test]
    fn from_iter_nullable() {
        let input = vec![Some(vec![1]), None, Some(vec![2, 3]), Some(vec![4])];
        let array = input
            .into_iter()
            .collect::<VariableSizeListArray<FixedSizePrimitiveArray<u8>, true>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0.data.0, &[1, 2, 3, 4]);
        assert_eq!(array.0.offsets.bitmap_ref().is_valid(0), Some(true));
        assert_eq!(array.0.offsets.bitmap_ref().is_null(1), Some(true));
        assert_eq!(array.0.offsets.bitmap_ref().is_valid(2), Some(true));
        assert_eq!(array.0.offsets.bitmap_ref().is_valid(3), Some(true));
        assert_eq!(array.0.offsets.as_ref().as_slice(), &[0, 1, 1, 3, 4]);
    }

    #[test]
    fn from_iter_nested() {
        let input = vec![vec![vec![1, 2, 3], vec![1, 2, 3]], vec![vec![4, 5, 6]]];
        let array = input
            .into_iter()
            .collect::<VariableSizeListArray<VariableSizeListArray<FixedSizePrimitiveArray<u8>>>>();
        assert_eq!(array.0.data.0.data.0, &[1, 2, 3, 1, 2, 3, 4, 5, 6]);
        assert_eq!(array.0.offsets, &[0, 2, 3]);
        assert_eq!(array.0.data.0.offsets, &[0, 3, 6, 9]);

        let input_3 = vec![vec![
            vec![vec![1, 2, 3], vec![1, 2, 3]],
            vec![vec![4, 5, 6]],
        ]];
        let array_3 = input_3.into_iter().collect::<VariableSizeListArray<
            VariableSizeListArray<VariableSizeListArray<FixedSizePrimitiveArray<u8>>>,
        >>();
        assert_eq!(array_3.0.data.0.data.0.data.0, &[1, 2, 3, 1, 2, 3, 4, 5, 6]);
        assert_eq!(array_3.0.offsets, &[0, 2]);
        assert_eq!(array_3.0.data.0.offsets, &[0, 2, 3]);
        assert_eq!(array_3.0.data.0.data.0.offsets, &[0, 3, 6, 9]);
    }

    #[test]
    fn from_iter_nested_nullable() {
        let input = vec![
            None,
            Some(vec![
                None,
                Some(vec![None, Some(vec![1, 2, 3]), Some(vec![1, 2, 3])]),
                Some(vec![None, Some(vec![4, 5, 6])]),
            ]),
        ];
        let array = input.into_iter().collect::<VariableSizeListArray<
            VariableSizeListArray<VariableSizeListArray<FixedSizePrimitiveArray<u8>, true>, true>,
            true,
        >>();
        assert_eq!(array.0.data.0.data.0.data.0, &[1, 2, 3, 1, 2, 3, 4, 5, 6]);
        assert_eq!(array.0.offsets.as_ref(), &[0, 0, 3]);
        assert_eq!(array.0.data.0.offsets.as_ref(), &[0, 0, 3, 5]);
        assert_eq!(array.0.data.0.data.0.offsets.as_ref(), &[0, 0, 3, 6, 6, 9]);
        assert_eq!(array.is_null(0), Some(true));
        assert_eq!(array.is_valid(1), Some(true));
        assert_eq!(array.0.data.is_null(0), Some(true));
        assert_eq!(array.0.data.is_valid(1), Some(true));
        assert_eq!(array.0.data.0.data.is_null(0), Some(true));
        assert_eq!(array.0.data.0.data.is_valid(1), Some(true));
        assert_eq!(array.0.data.0.data.0.is_null(0), Some(true));
        assert_eq!(array.0.data.0.data.0.is_valid(1), Some(true));

        let input_3 = vec![
            None,
            Some(vec![
                None,
                Some(vec![
                    None,
                    Some(vec![None, Some(2), Some(3)]),
                    Some(vec![None, Some(2), Some(3)]),
                ]),
                Some(vec![None, Some(vec![None, Some(5), Some(6)])]),
            ]),
        ];
        let array_3 = input_3.into_iter().collect::<VariableSizeListArray<
            VariableSizeListArray<
                VariableSizeListArray<FixedSizePrimitiveArray<u8, true>, true>,
                true,
            >,
            true,
        >>();
        assert_eq!(
            array_3.0.data.0.data.0.data.0.as_ref(),
            &[
                u8::default(),
                2,
                3,
                u8::default(),
                2,
                3,
                u8::default(),
                5,
                6
            ]
        );
        assert_eq!(array_3.0.offsets.as_ref(), &[0, 0, 3]);
        assert_eq!(array_3.0.data.0.offsets.as_ref(), &[0, 0, 3, 5]);
        assert_eq!(
            array_3.0.data.0.data.0.offsets.as_ref(),
            &[0, 0, 3, 6, 6, 9]
        );
        assert_eq!(array_3.is_null(0), Some(true));
        assert_eq!(array_3.is_valid(1), Some(true));
        assert_eq!(array_3.0.data.is_null(0), Some(true));
        assert_eq!(array_3.0.data.is_valid(1), Some(true));
        assert_eq!(array_3.0.data.0.data.is_null(0), Some(true));
        assert_eq!(array_3.0.data.0.data.is_valid(1), Some(true));
        assert_eq!(array_3.0.data.0.data.0.is_null(0), Some(true));
        assert_eq!(array_3.0.data.0.data.0.is_valid(1), Some(true));
        assert_eq!(array_3.0.data.0.data.0.data.0.is_null(0), Some(true));
        assert_eq!(array_3.0.data.0.data.0.data.0.is_valid(1), Some(true));
    }
}
