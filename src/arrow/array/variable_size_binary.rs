//! Interop with [`arrow-rs`] binary array.

use std::sync::Arc;

use arrow_array::OffsetSizeTrait;
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow_schema::{DataType, Field};

use crate::{
    array::{FixedSizePrimitiveArray, VariableSizeBinaryArray},
    arrow::OffsetElement,
    bitmap::Bitmap,
    buffer::BufferType,
    nullable::Nullable,
    offset::Offset,
    validity::{Nullability, Validity},
};

impl<const NULLABLE: bool, OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    crate::arrow::Array for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Vec<u8>: Nullability<NULLABLE>,
{
    type Array = arrow_array::GenericBinaryArray<OffsetItem>;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(
            name,
            if OffsetItem::LARGE {
                DataType::LargeBinary
            } else {
                DataType::Binary
            },
            NULLABLE,
        )
    }
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<Arc<dyn arrow_array::Array>> for VariableSizeBinaryArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Self: From<arrow_array::GenericBinaryArray<OffsetItem>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::GenericBinaryArray::<OffsetItem>::from(
            value.to_data(),
        ))
    }
}

impl<OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<VariableSizeBinaryArray<false, OffsetItem, Buffer>> for Arc<dyn arrow_array::Array>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, false, Buffer>: Into<arrow_buffer::ScalarBuffer<u8>>,
{
    fn from(value: VariableSizeBinaryArray<false, OffsetItem, Buffer>) -> Self {
        let array: arrow_array::GenericBinaryArray<OffsetItem> = value.into();
        Arc::new(array)
    }
}

impl<OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<VariableSizeBinaryArray<true, OffsetItem, Buffer>> for Arc<dyn arrow_array::Array>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, false, Buffer>: Into<arrow_buffer::ScalarBuffer<u8>>,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn from(value: VariableSizeBinaryArray<true, OffsetItem, Buffer>) -> Self {
        let array: arrow_array::GenericBinaryArray<OffsetItem> = value.into();
        Arc::new(array)
    }
}

impl<OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<VariableSizeBinaryArray<false, OffsetItem, Buffer>>
    for arrow_array::GenericBinaryArray<OffsetItem>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, false, Buffer>: Into<arrow_buffer::ScalarBuffer<u8>>,
{
    fn from(value: VariableSizeBinaryArray<false, OffsetItem, Buffer>) -> Self {
        arrow_array::GenericBinaryArray::new(
            // Safety:
            // - The narrow offset buffer contains valid offset data
            unsafe { OffsetBuffer::new_unchecked(value.0.offsets.into()) },
            value.0.data.into().into_inner(),
            None,
        )
    }
}

impl<OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<VariableSizeBinaryArray<true, OffsetItem, Buffer>>
    for arrow_array::GenericBinaryArray<OffsetItem>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, false, Buffer>: Into<arrow_buffer::ScalarBuffer<u8>>,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn from(value: VariableSizeBinaryArray<true, OffsetItem, Buffer>) -> Self {
        arrow_array::GenericBinaryArray::new(
            // Safety:
            // - The narrow offset buffer contains valid offset data
            unsafe { OffsetBuffer::new_unchecked(value.0.offsets.data.into()) },
            value.0.data.into().into_inner(),
            Some(value.0.offsets.validity.into()),
        )
    }
}

/// Panics when there are nulls
impl<OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<arrow_array::GenericBinaryArray<OffsetItem>>
    for VariableSizeBinaryArray<false, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: From<ScalarBuffer<u8>>,
    <Buffer as BufferType>::Buffer<OffsetItem>: From<ScalarBuffer<OffsetItem>>,
{
    fn from(value: arrow_array::GenericBinaryArray<OffsetItem>) -> Self {
        let (offsets, values, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => VariableSizeBinaryArray(Offset {
                data: ScalarBuffer::from(values).into(),
                offsets: offsets.into_inner().into(),
            }),
        }
    }
}

impl<OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<arrow_array::GenericBinaryArray<OffsetItem>>
    for VariableSizeBinaryArray<true, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: From<ScalarBuffer<u8>>,
    <Buffer as BufferType>::Buffer<OffsetItem>: From<ScalarBuffer<OffsetItem>>,
    Bitmap<Buffer>: From<NullBuffer> + FromIterator<bool>,
{
    fn from(value: arrow_array::GenericBinaryArray<OffsetItem>) -> Self {
        let (offsets_buffer, values, nulls_opt) = value.into_parts();
        let data = ScalarBuffer::from(values).into();
        let offsets = offsets_buffer.into_inner().into();
        match nulls_opt {
            Some(null_buffer) => VariableSizeBinaryArray(Offset {
                data,
                offsets: Nullable {
                    data: offsets,
                    validity: null_buffer.into(),
                },
            }),
            None => VariableSizeBinaryArray::<false, OffsetItem, Buffer>(Offset { data, offsets })
                .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{array::VariableSizeBinaryArray, bitmap::ValidityBitmap};

    fn input() -> [Vec<u8>; 3] {
        [vec![0, 1, 2], vec![3], vec![]]
    }

    fn input_nullable() -> [Option<Vec<u8>>; 3] {
        [Some(vec![0, 1, 2]), Some(vec![3]), None]
    }

    #[test]
    fn from() {
        let vsb_array = input().into_iter().collect::<VariableSizeBinaryArray>();
        assert_eq!(
            arrow_array::BinaryArray::from(vsb_array)
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            input()
        );

        let vsb_array_nullable = input_nullable()
            .into_iter()
            .collect::<VariableSizeBinaryArray<true, i64>>();
        assert_eq!(
            arrow_array::GenericBinaryArray::<i64>::from(vsb_array_nullable)
                .into_iter()
                .map(|o| o.map(<[u8]>::to_vec))
                .collect::<Vec<_>>(),
            input_nullable()
        );
    }

    #[test]
    fn into_nullable() {
        let vsb_array = input()
            .into_iter()
            .map(Option::Some)
            .collect::<arrow_array::BinaryArray>();
        assert!(
            !VariableSizeBinaryArray::<true, i32, crate::arrow::buffer::ScalarBuffer>::from(
                vsb_array
            )
            .any_null()
        );
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let vsb_array_nullable = input_nullable()
            .into_iter()
            .collect::<arrow_array::BinaryArray>();
        let _: VariableSizeBinaryArray<false, i32, crate::arrow::buffer::ScalarBuffer> =
            vsb_array_nullable.into();
    }

    #[test]
    fn into() {
        let vsb_array = input()
            .into_iter()
            .map(Option::Some)
            .collect::<arrow_array::BinaryArray>();
        let _: VariableSizeBinaryArray<false, i32, crate::arrow::buffer::ScalarBuffer> =
            vsb_array.into();
        // todo(mbrobbel): intoiterator for Binaryarray

        let vsb_array_nullable = input_nullable()
            .into_iter()
            .collect::<arrow_array::BinaryArray>();
        let _: VariableSizeBinaryArray<true, i32, crate::arrow::buffer::ScalarBuffer> =
            vsb_array_nullable.into();
        // todo(mbrobbel): intoiterator for Binaryarray
    }
}
