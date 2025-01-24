//! Interop with [`arrow-rs`] string array.

use std::sync::Arc;

use arrow_array::OffsetSizeTrait;
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow_schema::{DataType, Field};

use crate::{
    array::{FixedSizePrimitiveArray, StringArray, VariableSizeBinaryArray},
    arrow::Offset,
    bitmap::Bitmap,
    buffer::BufferType,
    nullability::{NonNullable, Nullability, Nullable},
    offset::Offsets,
    validity::Validity,
};

impl<Nullable: Nullability, OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    crate::arrow::Array for StringArray<Nullable, OffsetItem, Buffer>
{
    type Array = arrow_array::GenericStringArray<OffsetItem>;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(name, Self::data_type(), Nullable::NULLABLE)
    }

    fn data_type() -> arrow_schema::DataType {
        if OffsetItem::LARGE {
            DataType::LargeUtf8
        } else {
            DataType::Utf8
        }
    }
}

impl<Nullable: Nullability, OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<Arc<dyn arrow_array::Array>> for StringArray<Nullable, OffsetItem, Buffer>
where
    Self: From<arrow_array::GenericStringArray<OffsetItem>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::GenericStringArray::<OffsetItem>::from(
            value.to_data(),
        ))
    }
}

impl<OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<StringArray<NonNullable, OffsetItem, Buffer>> for Arc<dyn arrow_array::Array>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<u8>>,
{
    fn from(value: StringArray<NonNullable, OffsetItem, Buffer>) -> Self {
        let array: arrow_array::GenericStringArray<OffsetItem> = value.into();
        Arc::new(array)
    }
}

impl<OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<StringArray<Nullable, OffsetItem, Buffer>> for Arc<dyn arrow_array::Array>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<u8>>,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn from(value: StringArray<Nullable, OffsetItem, Buffer>) -> Self {
        let array: arrow_array::GenericStringArray<OffsetItem> = value.into();
        Arc::new(array)
    }
}

impl<OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<StringArray<NonNullable, OffsetItem, Buffer>>
    for arrow_array::GenericStringArray<OffsetItem>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<u8>>,
{
    fn from(value: StringArray<NonNullable, OffsetItem, Buffer>) -> Self {
        arrow_array::GenericStringArray::new(
            // Safety:
            // - The narrow offfset buffer contains valid offset data
            unsafe { OffsetBuffer::new_unchecked(value.0 .0.offsets.into()) },
            value.0 .0.data.into().into_inner(),
            None,
        )
    }
}

impl<OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<StringArray<Nullable, OffsetItem, Buffer>> for arrow_array::GenericStringArray<OffsetItem>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<u8>>,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn from(value: StringArray<Nullable, OffsetItem, Buffer>) -> Self {
        arrow_array::GenericStringArray::new(
            // Safety:
            // - The narrow offfset buffer contains valid offset data
            unsafe { OffsetBuffer::new_unchecked(value.0 .0.offsets.data.into()) },
            value.0 .0.data.into().into_inner(),
            Some(value.0 .0.offsets.validity.into()),
        )
    }
}

/// Panics when there are nulls
impl<OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<arrow_array::GenericStringArray<OffsetItem>>
    for StringArray<NonNullable, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: From<ScalarBuffer<u8>>,
    <Buffer as BufferType>::Buffer<OffsetItem>: From<ScalarBuffer<OffsetItem>>,
{
    fn from(value: arrow_array::GenericStringArray<OffsetItem>) -> Self {
        let (offsets, values, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => StringArray(VariableSizeBinaryArray(Offsets {
                data: ScalarBuffer::from(values).into(),
                offsets: offsets.into_inner().into(),
            })),
        }
    }
}

impl<OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<arrow_array::GenericStringArray<OffsetItem>> for StringArray<Nullable, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: From<ScalarBuffer<u8>>,
    <Buffer as BufferType>::Buffer<OffsetItem>: From<ScalarBuffer<OffsetItem>>,
    Bitmap<Buffer>: From<NullBuffer> + FromIterator<bool>,
{
    fn from(value: arrow_array::GenericStringArray<OffsetItem>) -> Self {
        let (offsets_buffer, values, nulls_opt) = value.into_parts();
        let data = ScalarBuffer::from(values).into();
        let offsets = offsets_buffer.into_inner().into();
        match nulls_opt {
            Some(null_buffer) => StringArray(VariableSizeBinaryArray(Offsets {
                data,
                offsets: Validity {
                    data: offsets,
                    validity: null_buffer.into(),
                },
            })),
            None => {
                StringArray::<NonNullable, OffsetItem, Buffer>(VariableSizeBinaryArray(Offsets {
                    data,
                    offsets,
                }))
                .into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{array::StringArray, bitmap::ValidityBitmap, NonNullable, Nullable};

    const INPUT: [&str; 3] = ["hello", "world", "!"];
    const INPUT_NULLABLE: [Option<&str>; 3] = [Some("hello"), None, Some("!")];

    #[test]
    fn from() {
        let string_array = INPUT.into_iter().collect::<StringArray>();
        assert_eq!(
            arrow_array::StringArray::from(string_array)
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            INPUT
        );

        let string_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<StringArray<Nullable, i64>>();
        assert_eq!(
            arrow_array::GenericStringArray::<i64>::from(string_array_nullable)
                .into_iter()
                .collect::<Vec<_>>(),
            INPUT_NULLABLE
        );
    }

    #[test]
    fn into_nullable() {
        let string_array = INPUT
            .into_iter()
            .map(ToOwned::to_owned)
            .map(Option::Some)
            .collect::<arrow_array::StringArray>();
        assert!(
            !StringArray::<Nullable, i32, crate::arrow::buffer::ScalarBuffer>::from(string_array)
                .any_null()
        );
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let string_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<arrow_array::StringArray>();
        let _: StringArray<NonNullable, i32, crate::arrow::buffer::ScalarBuffer> =
            string_array_nullable.into();
    }

    #[test]
    fn into() {
        let string_array = INPUT
            .into_iter()
            .map(ToOwned::to_owned)
            .map(Option::Some)
            .collect::<arrow_array::StringArray>();
        let _: StringArray<NonNullable, i32, crate::arrow::buffer::ScalarBuffer> =
            string_array.into();
        // todo(mbrobbel): intoiterator for stringarray

        let string_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<arrow_array::StringArray>();
        let _: StringArray<Nullable, i32, crate::arrow::buffer::ScalarBuffer> =
            string_array_nullable.into();
        // todo(mbrobbel): intoiterator for stringarray
    }
}
