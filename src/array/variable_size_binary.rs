//! Variable-size binary elements.

use crate::{
    buffer::{BufferType, VecBuffer},
    offset::OffsetElement,
    validity::Validity,
    Length,
};

use super::{Array, FixedSizePrimitiveArray, VariableSizeListArray};

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
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>;

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Array
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
{
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Default
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
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
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    VariableSizeListArray<FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, OffsetItem, Buffer>:
        Extend<T>,
    // Offset<<Buffer as BufferType>::Buffer<u8>, NULLABLE, OffsetItem, Buffer>: Extend<T>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<T, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> FromIterator<T>
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    VariableSizeListArray<FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, OffsetItem, Buffer>:
        FromIterator<T>,
    // Offset<<Buffer as BufferType>::Buffer<u8>, NULLABLE, OffsetItem, Buffer>: FromIterator<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Length
    for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    VariableSizeListArray<FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, OffsetItem, Buffer>:
        Length,
    // Offset<<Buffer as BufferType>::Buffer<u8>, NULLABLE, OffsetItem, Buffer>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        let input = vec![vec![1], vec![], vec![2, 3], vec![4]];
        let array = input.into_iter().collect::<VariableSizeBinaryArray>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0 .0.data.0, &[1, 2, 3, 4]);
        assert_eq!(array.0 .0.offsets, &[0, 1, 1, 3, 4]);

        let input = vec![Some(vec![1]), None, Some(vec![2, 3]), Some(vec![4])];
        let array = input.into_iter().collect::<VariableSizeBinaryArray<true>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0 .0.data.as_ref().0, &[1, 0, 2, 3, 4]);
        assert_eq!(array.0 .0.offsets, &[0, 1, 1, 3, 4]);
    }
}
