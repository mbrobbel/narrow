//! Variable-size binary elements.

use super::{Array, FixedSizePrimitiveArray, VariableSizeListArray};
use crate::{
    Index, Length,
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{Buffer, BufferType, VecBuffer},
    nullability::{NonNullable, Nullability, Nullable},
    offset::{Offset, Offsets},
};

/// Variable-size binary elements.
pub struct VariableSizeBinaryArray<
    Nullable: Nullability = NonNullable,
    OffsetItem: Offset = i32,
    Buffer: BufferType = VecBuffer,
>(pub Offsets<FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, OffsetItem, Buffer>);

/// Variable-size binary elements, using `i32` offset values.
pub type BinaryArray<Nullable = NonNullable, Buffer = VecBuffer> =
    VariableSizeBinaryArray<Nullable, i32, Buffer>;

/// Variable-size binary elements, using `i64` offset values.
pub type LargeBinaryArray<Nullable = NonNullable, Buffer = VecBuffer> =
    VariableSizeBinaryArray<Nullable, i64, Buffer>;

impl<Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Array
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
{
    type Item = Nullable::Item<Vec<u8>>;
}

impl<Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Clone
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
where
    Offsets<FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, OffsetItem, Buffer>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Default
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
where
    Offsets<FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, OffsetItem, Buffer>:
        Default,
{
    fn default() -> Self {
        Self(Offsets::default())
    }
}

impl<T, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Extend<T>
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
where
    Offsets<FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, OffsetItem, Buffer>:
        Extend<T>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType>
    From<
        VariableSizeListArray<
            FixedSizePrimitiveArray<u8, NonNullable, Buffer>,
            Nullable,
            OffsetItem,
            Buffer,
        >,
    > for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
{
    fn from(
        value: VariableSizeListArray<
            FixedSizePrimitiveArray<u8, NonNullable, Buffer>,
            Nullable,
            OffsetItem,
            Buffer,
        >,
    ) -> Self {
        Self(value.0)
    }
}

impl<OffsetItem: Offset, Buffer: BufferType>
    From<VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer>>
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
where
    Offsets<FixedSizePrimitiveArray<u8, NonNullable, Buffer>, NonNullable, OffsetItem, Buffer>:
        Into<
            Offsets<FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, OffsetItem, Buffer>,
        >,
{
    fn from(value: VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer>) -> Self {
        Self(value.0.into())
    }
}

// impl<Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType>
//     From<StringArray<Nullable, OffsetItem, Buffer>>
//     for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
// {
//     fn from(value: StringArray<Nullable, OffsetItem, Buffer>) -> Self {
//         Self(value.0 .0)
//     }
// }

impl<T, Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> FromIterator<T>
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
where
    T: IntoIterator,
    Offsets<FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, OffsetItem, Buffer>:
        FromIterator<<T as IntoIterator>::IntoIter>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(IntoIterator::into_iter).collect())
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> Index
    for VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Index,
{
    type Item<'a>
        = &'a [u8]
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

impl<OffsetItem: Offset, Buffer: BufferType> Index
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Index,
{
    type Item<'a>
        = Option<&'a [u8]>
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

impl<Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> IntoIterator
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
where
    Offsets<FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, OffsetItem, Buffer>:
        IntoIterator,
{
    type Item = <Offsets<
        FixedSizePrimitiveArray<u8, NonNullable, Buffer>,
        Nullable,
        OffsetItem,
        Buffer,
    > as IntoIterator>::Item;
    type IntoIter = <Offsets<
        FixedSizePrimitiveArray<u8, NonNullable, Buffer>,
        Nullable,
        OffsetItem,
        Buffer,
    > as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<Nullable: Nullability, OffsetItem: Offset, Buffer: BufferType> Length
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
where
    Offsets<FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, OffsetItem, Buffer>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> BitmapRef
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
{
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> BitmapRefMut
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
{
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<OffsetItem: Offset, Buffer: BufferType> ValidityBitmap
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::BufferRef;
    use std::mem;

    // #[test]
    // TODO(mbrobbel): re-enable
    // fn from_variable_size_binary() {
    //     let input: [Vec<u8>; 4] = [vec![0, 1, 2], vec![3], vec![], vec![4, 5]];
    //     let array = input
    //         .into_iter()
    //         .map(VariableSizeBinary)
    //         .collect::<VariableSizeBinaryArray>();
    //     assert_eq!(array.len(), 4);
    //     assert_eq!(array.0.data.0, &[0, 1, 2, 3, 4, 5]);
    //     assert_eq!(array.0.offsets, &[0, 3, 4, 4, 6]);
    // }

    #[test]
    fn from_iter() {
        let input: [&[u8]; 4] = [&[1], &[2, 3], &[4, 5, 6], &[7, 8, 9, 0]];
        let array = input.into_iter().collect::<VariableSizeBinaryArray>();
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
    fn into_iter() {
        let input: [&[u8]; 4] = [&[1], &[2, 3], &[4, 5, 6], &[7, 8, 9, 0]];
        let array = input.into_iter().collect::<VariableSizeBinaryArray>();
        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(output, input);

        let input_vec = vec![vec![1], vec![], vec![2, 3], vec![4]];
        let array_vec = input_vec
            .clone()
            .into_iter()
            .collect::<VariableSizeBinaryArray>();
        let output_vec = array_vec.into_iter().collect::<Vec<_>>();
        assert_eq!(output_vec, input_vec);
    }

    #[test]
    fn from_iter_nullable() {
        let input: [Option<&[u8]>; 4] = [Some(&[1]), None, Some(&[4, 5, 6]), Some(&[7, 8, 9, 0])];
        let array = input
            .into_iter()
            .collect::<VariableSizeBinaryArray<Nullable>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0.data.0, &[1, 4, 5, 6, 7, 8, 9, 0]);
        assert_eq!(array.0.offsets.as_ref(), &[0, 1, 1, 4, 8]);
        assert_eq!(array.0.offsets.bitmap_ref().buffer_ref(), &[0b000_01101]);

        let input_vec = vec![Some(vec![1]), None, Some(vec![2, 3]), Some(vec![4])];
        let array_vec = input_vec
            .into_iter()
            .collect::<VariableSizeBinaryArray<Nullable>>();
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
    fn into_iter_nullable() {
        let input: [Option<&[u8]>; 4] = [Some(&[1]), None, Some(&[4, 5, 6]), Some(&[7, 8, 9, 0])];
        let array = input
            .into_iter()
            .collect::<VariableSizeBinaryArray<Nullable>>();
        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(output, input.map(|opt| opt.map(<[u8]>::to_vec)));

        let input_vec = vec![Some(vec![1]), None, Some(vec![2, 3]), Some(vec![4])];
        let array_vec = input_vec
            .clone()
            .into_iter()
            .collect::<VariableSizeBinaryArray<Nullable>>();
        let output_vec = array_vec.into_iter().collect::<Vec<_>>();
        assert_eq!(output_vec, input_vec);
    }

    #[cfg(feature = "derive")]
    #[test]
    fn with_derive() {
        use crate::array::{StructArray, VariableSizeBinary};

        #[derive(crate::ArrayType, Clone, Debug, PartialEq)]
        struct Foo {
            a: Option<Vec<VariableSizeBinary>>,
        }

        let input = [Foo { a: None }];
        let array = input.clone().into_iter().collect::<StructArray<Foo>>();
        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input.as_slice(), output);
    }

    #[test]
    fn index() {
        let input: [&[u8]; 4] = [&[1], &[2, 3], &[4, 5, 6], &[7, 8, 9, 0]];
        let array = input.into_iter().collect::<VariableSizeBinaryArray>();
        assert_eq!(array.index_checked(0), &[1]);
        assert_eq!(array.index_checked(1), &[2, 3]);
        assert_eq!(array.index_checked(2), &[4, 5, 6]);
        assert_eq!(array.index_checked(3), &[7, 8, 9, 0]);
        assert!(array.index(4).is_none());
    }

    // #[test]
    // TODO(mbrobbel):re-enable
    // fn convert() {
    //     let input = vec![Some("a".to_owned()), None, Some("b".to_owned())];
    //     let array = input.into_iter().collect::<StringArray<Nullable>>();
    //     let variable_size_binary: VariableSizeBinaryArray<Nullable> = array.into();
    //     assert_eq!(variable_size_binary.len(), 3);
    // }

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
            mem::size_of::<BinaryArray<Nullable>>(),
            mem::size_of::<Vec<u8>>() + mem::size_of::<Vec<i32>>() + mem::size_of::<Bitmap>()
        );
        assert_eq!(
            mem::size_of::<LargeBinaryArray<Nullable>>(),
            mem::size_of::<Vec<u8>>() + mem::size_of::<Vec<i64>>() + mem::size_of::<Bitmap>()
        );
    }
}
