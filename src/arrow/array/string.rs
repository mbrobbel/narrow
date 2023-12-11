//! Interop with [`arrow-rs`] string array.

use std::sync::Arc;

use arrow_array::OffsetSizeTrait;
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow_schema::{DataType, Field};

use crate::{
    array::{FixedSizePrimitiveArray, StringArray, VariableSizeBinaryArray},
    arrow::ArrowArray,
    bitmap::Bitmap,
    buffer::BufferType,
    nullable::Nullable,
    offset::{Offset, OffsetElement},
    validity::Validity,
};

impl<const NULLABLE: bool, OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    ArrowArray for StringArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
{
    type Array = arrow_array::GenericStringArray<OffsetItem>;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(name, DataType::Utf8, NULLABLE)
    }
}

impl<const NULLABLE: bool, OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<Arc<dyn arrow_array::Array>> for StringArray<NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Self: From<arrow_array::GenericStringArray<OffsetItem>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::GenericStringArray::<OffsetItem>::from(
            value.to_data(),
        ))
    }
}

impl<OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<StringArray<false, OffsetItem, Buffer>> for arrow_array::GenericStringArray<OffsetItem>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, false, Buffer>: Into<arrow_buffer::Buffer>,
{
    fn from(value: StringArray<false, OffsetItem, Buffer>) -> Self {
        arrow_array::GenericStringArray::new(
            // Safety:
            // - The narrow offfset buffer contains valid offset data
            unsafe { OffsetBuffer::new_unchecked(value.0 .0.offsets.into()) },
            value.0 .0.data.into(),
            None,
        )
    }
}

impl<OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<StringArray<true, OffsetItem, Buffer>> for arrow_array::GenericStringArray<OffsetItem>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    FixedSizePrimitiveArray<u8, false, Buffer>: Into<arrow_buffer::Buffer>,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn from(value: StringArray<true, OffsetItem, Buffer>) -> Self {
        arrow_array::GenericStringArray::new(
            // Safety:
            // - The narrow offfset buffer contains valid offset data
            unsafe { OffsetBuffer::new_unchecked(value.0 .0.offsets.data.into()) },
            value.0 .0.data.into(),
            Some(value.0 .0.offsets.validity.into()),
        )
    }
}

/// Panics when there are nulls
impl<OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<arrow_array::GenericStringArray<OffsetItem>> for StringArray<false, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: From<arrow_buffer::Buffer>,
    <Buffer as BufferType>::Buffer<OffsetItem>: From<ScalarBuffer<OffsetItem>>,
{
    fn from(value: arrow_array::GenericStringArray<OffsetItem>) -> Self {
        let (offsets, values, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => StringArray(VariableSizeBinaryArray(Offset {
                data: values.into(),
                offsets: offsets.into_inner().into(),
            })),
        }
    }
}

/// Panics when there are no nulls
impl<OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<arrow_array::GenericStringArray<OffsetItem>> for StringArray<true, OffsetItem, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: From<arrow_buffer::Buffer>,
    <Buffer as BufferType>::Buffer<OffsetItem>: From<ScalarBuffer<OffsetItem>>,
    Bitmap<Buffer>: From<NullBuffer>,
{
    fn from(value: arrow_array::GenericStringArray<OffsetItem>) -> Self {
        let (offsets, values, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(null_buffer) => StringArray(VariableSizeBinaryArray(Offset {
                data: values.into(),
                offsets: Nullable {
                    data: offsets.into_inner().into(),
                    validity: null_buffer.into(),
                },
            })),
            None => panic!("expected array with a null buffer"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::i64;

    use crate::{array::StringArray, arrow::scalar_buffer::ArrowScalarBuffer};

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
            .collect::<StringArray<true, i64>>();
        assert_eq!(
            arrow_array::GenericStringArray::<i64>::from(string_array_nullable)
                .into_iter()
                .collect::<Vec<_>>(),
            INPUT_NULLABLE
        );
    }

    #[test]
    #[should_panic(expected = "expected array with a null buffer")]
    fn into_nullable() {
        let string_array = INPUT
            .into_iter()
            .map(ToOwned::to_owned)
            .map(Option::Some)
            .collect::<arrow_array::StringArray>();
        let _: StringArray<true, i32, ArrowScalarBuffer> = string_array.into();
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let string_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<arrow_array::StringArray>();
        let _: StringArray<false, i32, ArrowScalarBuffer> = string_array_nullable.into();
    }

    #[test]
    fn into() {
        let string_array = INPUT
            .into_iter()
            .map(ToOwned::to_owned)
            .map(Option::Some)
            .collect::<arrow_array::StringArray>();
        let _: StringArray<false, i32, ArrowScalarBuffer> = string_array.into();
        // todo(mbrobbel): intoiterator for stringarray

        let string_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<arrow_array::StringArray>();
        let _: StringArray<true, i32, ArrowScalarBuffer> = string_array_nullable.into();
        // todo(mbrobbel): intoiterator for stringarray
    }
}
