//! Interop with [`arrow-rs`] binary array.

use std::sync::Arc;

use arrow_array::OffsetSizeTrait;
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow_schema::{DataType, Field};

use crate::{
    array::{FixedSizePrimitiveArray, VariableSizeBinaryArray},
    arrow::Offset,
    bitmap::Bitmap,
    buffer::BufferType,
    nullability::{NonNullable, Nullability, Nullable},
    offset::Offsets,
    validity::Validity,
};

impl<Nullable: Nullability, OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    crate::arrow::Array for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
{
    type Array = arrow_array::GenericBinaryArray<OffsetItem>;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(name, Self::data_type(), Nullable::NULLABLE)
    }

    fn data_type() -> arrow_schema::DataType {
        if OffsetItem::LARGE {
            DataType::LargeBinary
        } else {
            DataType::Binary
        }
    }
}

impl<Nullable: Nullability, OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<Arc<dyn arrow_array::Array>> for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
where
    Self: From<arrow_array::GenericBinaryArray<OffsetItem>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::GenericBinaryArray::<OffsetItem>::from(
            value.to_data(),
        ))
    }
}


impl<OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType> Into<Arc<dyn arrow_array::Array>>
    for VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<u8>>,
{
    fn into(self) -> Arc<dyn arrow_array::Array> {
        let array: arrow_array::GenericBinaryArray<OffsetItem> = self.into();
        Arc::new(array)
    }
}


impl<OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType> Into<Arc<dyn arrow_array::Array>>
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<u8>>,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn into(self) -> Arc<dyn arrow_array::Array> {
        let array: arrow_array::GenericBinaryArray<OffsetItem> = self.into();
        Arc::new(array)
    }
}


impl<OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    Into<arrow_array::GenericBinaryArray<OffsetItem>>
    for VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<u8>>,
{
    fn into(self) -> arrow_array::GenericBinaryArray<OffsetItem> {
        arrow_array::GenericBinaryArray::new(
            // Safety:
            // - The narrow offset buffer contains valid offset data
            unsafe { OffsetBuffer::new_unchecked(self.0.offsets.into()) },
            self.0.data.into().into_inner(),
            None,
        )
    }
}


impl<OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    Into<arrow_array::GenericBinaryArray<OffsetItem>>
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<u8>>,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn into(self) -> arrow_array::GenericBinaryArray<OffsetItem> {
        arrow_array::GenericBinaryArray::new(
            // Safety:
            // - The narrow offset buffer contains valid offset data
            unsafe { OffsetBuffer::new_unchecked(self.0.offsets.data.into()) },
            self.0.data.into().into_inner(),
            Some(self.0.offsets.validity.into()),
        )
    }
}

/// Panics when there are nulls
impl<OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<arrow_array::GenericBinaryArray<OffsetItem>>
    for VariableSizeBinaryArray<NonNullable, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: From<ScalarBuffer<u8>>,
    <Buffer as BufferType>::Buffer<OffsetItem>: From<ScalarBuffer<OffsetItem>>,
{
    fn from(value: arrow_array::GenericBinaryArray<OffsetItem>) -> Self {
        let (offsets, values, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => VariableSizeBinaryArray(Offsets {
                data: ScalarBuffer::from(values).into(),
                offsets: offsets.into_inner().into(),
            }),
        }
    }
}

impl<OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<arrow_array::GenericBinaryArray<OffsetItem>>
    for VariableSizeBinaryArray<Nullable, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: From<ScalarBuffer<u8>>,
    <Buffer as BufferType>::Buffer<OffsetItem>: From<ScalarBuffer<OffsetItem>>,
    Bitmap<Buffer>: From<NullBuffer> + FromIterator<bool>,
{
    fn from(value: arrow_array::GenericBinaryArray<OffsetItem>) -> Self {
        let (offsets_buffer, values, nulls_opt) = value.into_parts();
        let data = ScalarBuffer::from(values).into();
        let offsets = offsets_buffer.into_inner().into();
        match nulls_opt {
            Some(null_buffer) => VariableSizeBinaryArray(Offsets {
                data,
                offsets: Validity {
                    data: offsets,
                    validity: null_buffer.into(),
                },
            }),
            None => VariableSizeBinaryArray::<NonNullable, OffsetItem, Buffer>(Offsets {
                data,
                offsets,
            })
            .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{array::VariableSizeBinaryArray, bitmap::ValidityBitmap, NonNullable, Nullable};

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
            Into::<arrow_array::BinaryArray>::into(vsb_array)
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            input()
        );

        let vsb_array_nullable = input_nullable()
            .into_iter()
            .collect::<VariableSizeBinaryArray<Nullable, i64>>();
        assert_eq!(
            Into::<arrow_array::GenericBinaryArray::<i64>>::into(vsb_array_nullable)
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
            !VariableSizeBinaryArray::<Nullable, i32, crate::arrow::buffer::ScalarBuffer>::from(
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
        let _: VariableSizeBinaryArray<NonNullable, i32, crate::arrow::buffer::ScalarBuffer> =
            vsb_array_nullable.into();
    }

    #[test]
    fn into() {
        let vsb_array = input()
            .into_iter()
            .map(Option::Some)
            .collect::<arrow_array::BinaryArray>();
        let _: VariableSizeBinaryArray<NonNullable, i32, crate::arrow::buffer::ScalarBuffer> =
            vsb_array.into();
        // todo(mbrobbel): intoiterator for Binaryarray

        let vsb_array_nullable = input_nullable()
            .into_iter()
            .collect::<arrow_array::BinaryArray>();
        let _: VariableSizeBinaryArray<Nullable, i32, crate::arrow::buffer::ScalarBuffer> =
            vsb_array_nullable.into();
        // todo(mbrobbel): intoiterator for Binaryarray
    }
}
