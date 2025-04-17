//!Array with variable-size list elements.

use crate::{
    Index, Length,
    array::Array,
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{BufferType, VecBuffer},
    nullability::{NonNullable, Nullability, Nullable},
    offset::{Offset, Offsets},
};
use std::fmt::{Debug, Formatter, Result};

/// Array with variable-size list elements.
pub struct VariableSizeListArray<
    T: Array, // todo(mbrobbel): move this bound?
    Nullable: Nullability = NonNullable,
    OffsetItem: Offset = i32,
    Buffer: BufferType = VecBuffer,
>(pub Offsets<T, Nullable, OffsetItem, Buffer>);

impl<T: Array, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Array
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
{
    type Item = Nullable::Item<Vec<T>>;
}

impl<T: Array, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Debug
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
where
    Offsets<T, Nullable, OffsetItem, Buffer>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_tuple("VariableSizeListArray")
            .field(&self.0)
            .finish()
    }
}

impl<T: Array, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Clone
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
where
    Offsets<T, Nullable, OffsetItem, Buffer>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Array, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Default
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
where
    Offsets<T, Nullable, OffsetItem, Buffer>: Default,
{
    fn default() -> Self {
        Self(Offsets::default())
    }
}

impl<T: Array, U, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Extend<U>
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
where
    Offsets<T, Nullable, OffsetItem, Buffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: Array, OffsetItem: Offset, Buffer: BufferType>
    From<VariableSizeListArray<T, NonNullable, OffsetItem, Buffer>>
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
where
    Offsets<T, NonNullable, OffsetItem, Buffer>: Into<Offsets<T, Nullable, OffsetItem, Buffer>>,
{
    fn from(value: VariableSizeListArray<T, NonNullable, OffsetItem, Buffer>) -> Self {
        Self(value.0.into())
    }
}

impl<T: Array, U, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> FromIterator<U>
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
where
    Offsets<T, Nullable, OffsetItem, Buffer>: FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: Array, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> IntoIterator
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
where
    Offsets<T, Nullable, OffsetItem, Buffer>: IntoIterator,
{
    type IntoIter = <Offsets<T, Nullable, OffsetItem, Buffer> as IntoIterator>::IntoIter;
    type Item = <Offsets<T, Nullable, OffsetItem, Buffer> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Array, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Index
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
where
    Offsets<T, Nullable, OffsetItem, Buffer>: Index,
{
    type Item<'a>
        = <Offsets<T, Nullable, OffsetItem, Buffer> as Index>::Item<'a>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.0.index_unchecked(index)
    }
}

impl<T: Array, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Length
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
where
    Offsets<T, Nullable, OffsetItem, Buffer>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Array, OffsetItem: Offset, Buffer: BufferType> BitmapRef
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
{
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<T: Array, OffsetItem: Offset, Buffer: BufferType> BitmapRefMut
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
{
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<T: Array, OffsetItem: Offset, Buffer: BufferType> ValidityBitmap
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::array::FixedSizePrimitiveArray;

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
            .collect::<VariableSizeListArray<FixedSizePrimitiveArray<u8>, Nullable>>();
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
            VariableSizeListArray<
                VariableSizeListArray<FixedSizePrimitiveArray<u8>, Nullable>,
                Nullable,
            >,
            Nullable,
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
                VariableSizeListArray<FixedSizePrimitiveArray<u8, Nullable>, Nullable>,
                Nullable,
            >,
            Nullable,
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

    #[test]
    fn index() {
        let input = vec![vec![vec![1, 2, 3], vec![1, 2, 3]], vec![vec![4, 5, 6]]];
        let array = input
            .into_iter()
            .collect::<VariableSizeListArray<VariableSizeListArray<FixedSizePrimitiveArray<u8>>>>();
        assert_eq!(
            array
                .index_checked(0)
                .flatten()
                .copied()
                .collect::<Vec<_>>(),
            [1, 2, 3, 1, 2, 3]
        );
        assert_eq!(
            array
                .index_checked(1)
                .next()
                .expect("a value")
                .copied()
                .collect::<Vec<_>>(),
            [4, 5, 6]
        );
        assert!(array.index(2).is_none());
    }

    #[test]
    fn into_iter_nested() {
        let input = vec![
            vec![vec![1, 2, 3], vec![1, 2, 3]],
            vec![],
            vec![vec![], vec![]],
            vec![vec![4, 5, 6]],
        ];
        let array = input
            .clone()
            .into_iter()
            .collect::<VariableSizeListArray<VariableSizeListArray<FixedSizePrimitiveArray<u8>>>>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), input);
    }

    #[test]
    fn into_iter_nested_nullable() {
        let input = vec![
            None,
            Some(vec![]),
            Some(vec![None, Some(vec![])]),
            Some(vec![
                None,
                Some(vec![None, Some(vec![1, 2, 3]), Some(vec![1, 2, 3])]),
                Some(vec![None, Some(vec![4, 5, 6])]),
            ]),
        ];
        let array = input.clone().into_iter().collect::<VariableSizeListArray<
            VariableSizeListArray<
                VariableSizeListArray<FixedSizePrimitiveArray<u8>, Nullable>,
                Nullable,
            >,
            Nullable,
        >>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), input);
    }
}
