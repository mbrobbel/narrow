use super::Array;
use crate::{
    buffer::{BufferType, VecBuffer},
    offset::{Offset, OffsetElement},
    validity::Validity,
    Length,
};

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

impl<T: Array, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Default
    for VariableSizeListArray<T, NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Offset<T, NULLABLE, OffsetItem, Buffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Array, U, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Extend<U>
    for VariableSizeListArray<T, NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Offset<T, NULLABLE, OffsetItem, Buffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter)
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

        let input = vec![Some(vec![1]), None, Some(vec![2, 3]), Some(vec![4])];
        let array = input
            .into_iter()
            .collect::<VariableSizeListArray<FixedSizePrimitiveArray<u8>, true>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0.data.0, &[1, 2, 3, 4]);
        assert!(array.0.offsets.bitmap_ref().is_valid(0).unwrap());
        assert!(array.0.offsets.bitmap_ref().is_null(1).unwrap());
        assert!(array.0.offsets.bitmap_ref().is_valid(2).unwrap());
        assert!(array.0.offsets.bitmap_ref().is_valid(3).unwrap());
        assert_eq!(array.0.offsets.as_ref().as_slice(), &[0, 1, 1, 3, 4]);

        let input = vec![vec![vec![1, 2, 3], vec![1, 2, 3]], vec![vec![4, 5, 6]]];
        let array = input
            .into_iter()
            .collect::<VariableSizeListArray<VariableSizeListArray<FixedSizePrimitiveArray<u8>>>>();
        assert_eq!(array.0.data.0.data.0, &[1, 2, 3, 1, 2, 3, 4, 5, 6]);
        assert_eq!(array.0.offsets, &[0, 2, 3]);
        assert_eq!(array.0.data.0.offsets, &[0, 3, 6, 9]);

        let input = vec![vec![
            vec![vec![1, 2, 3], vec![1, 2, 3]],
            vec![vec![4, 5, 6]],
        ]];
        let array = input.into_iter().collect::<VariableSizeListArray<
            VariableSizeListArray<VariableSizeListArray<FixedSizePrimitiveArray<u8>>>,
        >>();
        assert_eq!(array.0.data.0.data.0.data.0, &[1, 2, 3, 1, 2, 3, 4, 5, 6]);
        assert_eq!(array.0.offsets, &[0, 2]);
        assert_eq!(array.0.data.0.offsets, &[0, 2, 3]);
        assert_eq!(array.0.data.0.data.0.offsets, &[0, 3, 6, 9]);
    }
}
