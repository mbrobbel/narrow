//! Interop with `arrow-rs` fixed-sized binary array.

use std::sync::Arc;

use arrow_buffer::NullBuffer;
use arrow_schema::{DataType, Field};

use crate::{
    array::{FixedSizeBinaryArray, FixedSizeListArray, FixedSizePrimitiveArray},
    bitmap::Bitmap,
    buffer::BufferType,
    nullability::{NonNullable, Nullability, Nullable},
    validity::Validity,
};

impl<const N: usize, Nullable: Nullability, Buffer: BufferType> crate::arrow::Array
    for FixedSizeBinaryArray<N, Nullable, Buffer>
{
    type Array = arrow_array::FixedSizeBinaryArray;

    fn as_field(name: &str) -> Field {
        // todo(mbrobbel): const_assert
        assert!(N <= 0x7FFF_FFFF); // i32::MAX
        #[allow(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap
        )]
        Field::new(name, Self::data_type(), Nullable::NULLABLE)
    }

    fn data_type() -> arrow_schema::DataType {
        DataType::FixedSizeBinary(i32::try_from(N).expect("overflow"))
    }
}

impl<const N: usize, Nullable: Nullability, Buffer: BufferType> From<Arc<dyn arrow_array::Array>>
    for FixedSizeBinaryArray<N, Nullable, Buffer>
where
    Self: From<arrow_array::FixedSizeBinaryArray>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::FixedSizeBinaryArray::from(value.to_data()))
    }
}

impl<const N: usize, Buffer: BufferType> From<FixedSizeBinaryArray<N, NonNullable, Buffer>>
    for arrow_array::FixedSizeBinaryArray
where
    arrow_buffer::Buffer: From<FixedSizePrimitiveArray<u8, NonNullable, Buffer>>,
{
    fn from(value: FixedSizeBinaryArray<N, NonNullable, Buffer>) -> Self {
        // todo(mbrobbel): const_assert
        assert!(N <= 0x7FFF_FFFF); // i32::MAX
        #[allow(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap
        )]
        arrow_array::FixedSizeBinaryArray::new(
            i32::try_from(N).expect("overflow"),
            value.0 .0.into(),
            None,
        )
    }
}

impl<Nullable: Nullability, const N: usize, Buffer: BufferType>
    From<FixedSizeBinaryArray<N, Nullable, Buffer>> for Arc<dyn arrow_array::Array>
where
    arrow_array::FixedSizeBinaryArray: From<FixedSizeBinaryArray<N, Nullable, Buffer>>,
{
    fn from(value: FixedSizeBinaryArray<N, Nullable, Buffer>) -> Self {
        Arc::new(arrow_array::FixedSizeBinaryArray::from(value))
    }
}

impl<const N: usize, Buffer: BufferType> From<FixedSizeBinaryArray<N, Nullable, Buffer>>
    for arrow_array::FixedSizeBinaryArray
where
    arrow_buffer::Buffer: From<FixedSizePrimitiveArray<u8, NonNullable, Buffer>>,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn from(value: FixedSizeBinaryArray<N, Nullable, Buffer>) -> Self {
        // todo(mbrobbel): const_assert
        assert!(N <= 0x7FFF_FFFF); // i32::MAX
        #[allow(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap
        )]
        arrow_array::FixedSizeBinaryArray::new(
            i32::try_from(N).expect("overflow"),
            value.0 .0.data.into(),
            Some(value.0 .0.validity.into()),
        )
    }
}

/// Panics when there are nulls
impl<const N: usize, Buffer: BufferType> From<arrow_array::FixedSizeBinaryArray>
    for FixedSizeBinaryArray<N, NonNullable, Buffer>
where
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: From<arrow_buffer::ScalarBuffer<u8>>,
{
    fn from(value: arrow_array::FixedSizeBinaryArray) -> Self {
        let (n, values, nulls_opt) = value.into_parts();
        assert_eq!(N, n.try_into().expect("size to cast to usize"));
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => FixedSizeBinaryArray(FixedSizeListArray(
                arrow_buffer::ScalarBuffer::from(values).into(),
            )),
        }
    }
}

impl<const N: usize, Buffer: BufferType> From<arrow_array::FixedSizeBinaryArray>
    for FixedSizeBinaryArray<N, Nullable, Buffer>
where
    FixedSizePrimitiveArray<u8, NonNullable, Buffer>: From<arrow_buffer::ScalarBuffer<u8>>,
    Bitmap<Buffer>: From<NullBuffer> + FromIterator<bool>,
{
    fn from(value: arrow_array::FixedSizeBinaryArray) -> Self {
        let (n, values, nulls_opt) = value.into_parts();
        assert_eq!(N, n.try_into().expect("size to cast to usize"));
        let data = arrow_buffer::ScalarBuffer::from(values).into();
        match nulls_opt {
            Some(null_buffer) => FixedSizeBinaryArray(FixedSizeListArray(Validity {
                data,
                validity: null_buffer.into(),
            })),
            None => FixedSizeBinaryArray::<N, NonNullable, Buffer>(FixedSizeListArray(data)).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bitmap::ValidityBitmap;

    use super::*;

    const INPUT: [[u8; 2]; 3] = [[1, 2], [3, 4], [5, 6]];
    const INPUT_NULLABLE: [Option<[u8; 2]>; 3] = [Some([1, 2]), None, Some([5, 6])];

    #[test]
    fn from() {
        let fixed_size_binary_array = INPUT.into_iter().collect::<FixedSizeBinaryArray<2>>();
        assert_eq!(
            arrow_array::FixedSizeBinaryArray::from(fixed_size_binary_array)
                .iter()
                .flatten()
                .flatten()
                .copied()
                .collect::<Vec<_>>(),
            INPUT.into_iter().flatten().collect::<Vec<_>>()
        );

        let fixed_size_binary_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<FixedSizeBinaryArray<2, Nullable>>();
        assert_eq!(
            arrow_array::FixedSizeBinaryArray::from(fixed_size_binary_array_nullable)
                .iter()
                .flatten()
                .flatten()
                .copied()
                .collect::<Vec<_>>(),
            INPUT_NULLABLE
                .into_iter()
                .flatten()
                .flatten()
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn into_nullable() {
        let fixed_size_binary_array =
            arrow_array::FixedSizeBinaryArray::try_from_iter(INPUT.into_iter()).expect("");
        assert!(!FixedSizeBinaryArray::<2, Nullable>::from(fixed_size_binary_array).any_null());
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let fixed_size_binary_array_nullable =
            arrow_array::FixedSizeBinaryArray::from(vec![None, Some([1_u8, 2, 3].as_slice())]);
        let _ = FixedSizeBinaryArray::<3, NonNullable>::from(fixed_size_binary_array_nullable);
    }

    #[test]
    fn into() {
        let fixed_size_binary_array =
            arrow_array::FixedSizeBinaryArray::try_from_iter(INPUT.into_iter()).expect("");
        assert_eq!(
            FixedSizeBinaryArray::<2, NonNullable>::from(fixed_size_binary_array)
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            INPUT.into_iter().flatten().collect::<Vec<_>>()
        );

        let fixed_size_binary_array_nullable_input = INPUT_NULLABLE
            .into_iter()
            .collect::<FixedSizeBinaryArray<2, Nullable>>();
        let fixed_size_binary_array_nullable =
            arrow_array::FixedSizeBinaryArray::from(fixed_size_binary_array_nullable_input);
        assert_eq!(
            FixedSizeBinaryArray::<2, Nullable>::from(fixed_size_binary_array_nullable)
                .into_iter()
                .collect::<Vec<_>>(),
            INPUT_NULLABLE
        );
    }
}
