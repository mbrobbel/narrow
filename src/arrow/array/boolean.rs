//! Interop with `arrow-rs` boolean array.

use std::sync::Arc;

use crate::{
    array::BooleanArray, arrow::ArrowArray, bitmap::Bitmap, buffer::BufferType, nullable::Nullable,
    validity::Validity,
};
use arrow_buffer::{BooleanBuffer, NullBuffer};
use arrow_schema::{DataType, Field};

impl<const NULLABLE: bool, Buffer: BufferType> ArrowArray for BooleanArray<NULLABLE, Buffer>
where
    Bitmap<Buffer>: Validity<NULLABLE>,
{
    type Array = arrow_array::BooleanArray;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(name, DataType::Boolean, NULLABLE)
    }
}

impl<const NULLABLE: bool, Buffer: BufferType> From<Arc<dyn arrow_array::Array>>
    for BooleanArray<NULLABLE, Buffer>
where
    Bitmap<Buffer>: Validity<NULLABLE>,
    Self: From<arrow_array::BooleanArray>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::BooleanArray::from(value.to_data()))
    }
}

impl<Buffer: BufferType> From<BooleanArray<false, Buffer>> for arrow_array::BooleanArray
where
    Bitmap<Buffer>: Into<BooleanBuffer>,
{
    fn from(value: BooleanArray<false, Buffer>) -> Self {
        arrow_array::BooleanArray::new(value.0.into(), None)
    }
}

impl<Buffer: BufferType> From<BooleanArray<true, Buffer>> for arrow_array::BooleanArray
where
    Bitmap<Buffer>: Into<BooleanBuffer> + Into<NullBuffer>,
{
    fn from(value: BooleanArray<true, Buffer>) -> Self {
        arrow_array::BooleanArray::new(value.0.data.into(), Some(value.0.validity.into()))
    }
}

/// Panics when there are nulls
impl<Buffer: BufferType> From<arrow_array::BooleanArray> for BooleanArray<false, Buffer>
where
    Bitmap<Buffer>: From<BooleanBuffer>,
{
    fn from(value: arrow_array::BooleanArray) -> Self {
        let (boolean_buffer, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => BooleanArray(boolean_buffer.into()),
        }
    }
}

/// Panics when there are no nulls
// OR allocate one instead and use `TryFrom` conversion?
impl<Buffer: BufferType> From<arrow_array::BooleanArray> for BooleanArray<true, Buffer>
where
    Bitmap<Buffer>: From<BooleanBuffer> + From<NullBuffer>,
{
    fn from(value: arrow_array::BooleanArray) -> Self {
        let (boolean_buffer, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(null_buffer) => BooleanArray(Nullable {
                data: boolean_buffer.into(),
                validity: null_buffer.into(),
            }),
            None => panic!("expected array with a null buffer"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{array::BooleanArray, buffer::ArcBuffer};

    const INPUT: [bool; 4] = [true, true, false, true];
    const INPUT_NULLABLE: [Option<bool>; 4] = [Some(true), None, Some(false), Some(true)];

    #[test]
    fn from() {
        let boolean_array = INPUT.into_iter().collect::<BooleanArray>();
        assert_eq!(
            arrow_array::BooleanArray::from(boolean_array)
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            INPUT
        );

        let boolean_array_arc = INPUT
            .into_iter()
            .collect::<BooleanArray<false, ArcBuffer>>();
        assert_eq!(
            arrow_array::BooleanArray::from(boolean_array_arc)
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            INPUT
        );

        let boolean_array_nullable = INPUT_NULLABLE.into_iter().collect::<BooleanArray<true>>();
        assert_eq!(
            arrow_array::BooleanArray::from(boolean_array_nullable)
                .into_iter()
                .collect::<Vec<_>>(),
            INPUT_NULLABLE
        );
    }

    #[test]
    #[should_panic(expected = "expected array with a null buffer")]
    fn into_nullable() {
        let boolean_array = arrow_array::BooleanArray::from(INPUT.into_iter().collect::<Vec<_>>());
        let _ = BooleanArray::<true, crate::arrow::buffer::scalar_buffer::ArrowScalarBuffer>::from(
            boolean_array,
        );
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let boolean_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<arrow_array::BooleanArray>();
        let _ = BooleanArray::<false, crate::arrow::buffer::scalar_buffer::ArrowScalarBuffer>::from(
            boolean_array_nullable,
        );
    }

    #[test]
    fn into() {
        let boolean_array = arrow_array::BooleanArray::from(INPUT.into_iter().collect::<Vec<_>>());
        assert_eq!(
            BooleanArray::<false, crate::arrow::buffer::scalar_buffer::ArrowScalarBuffer>::from(
                boolean_array
            )
            .into_iter()
            .collect::<Vec<_>>(),
            INPUT
        );

        let boolean_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<arrow_array::BooleanArray>();
        assert_eq!(
            BooleanArray::<true, crate::arrow::buffer::scalar_buffer::ArrowScalarBuffer>::from(
                boolean_array_nullable
            )
            .into_iter()
            .collect::<Vec<_>>(),
            INPUT_NULLABLE
        );
    }
}
