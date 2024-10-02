//! Interop with `arrow-rs` fixed-sized binary array.

use std::sync::Arc;

use arrow_buffer::NullBuffer;
use arrow_schema::{DataType, Field};

use crate::{
    array::{FixedSizeBinaryArray, FixedSizeListArray, FixedSizePrimitiveArray},
    bitmap::Bitmap,
    buffer::BufferType,
    nullable::Nullable,
    validity::{Nullability, Validity},
};

impl<const N: usize, const NULLABLE: bool, Buffer: BufferType> crate::arrow::Array
    for FixedSizeBinaryArray<N, NULLABLE, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    [u8; N]: Nullability<NULLABLE>,
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
        Field::new(name, DataType::FixedSizeBinary(N as i32), NULLABLE)
    }
}

impl<const N: usize, const NULLABLE: bool, Buffer: BufferType> From<Arc<dyn arrow_array::Array>>
    for FixedSizeBinaryArray<N, NULLABLE, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    Self: From<arrow_array::FixedSizeBinaryArray>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::FixedSizeBinaryArray::from(value.to_data()))
    }
}

impl<const N: usize, Buffer: BufferType> From<FixedSizeBinaryArray<N, false, Buffer>>
    for arrow_array::FixedSizeBinaryArray
where
    arrow_buffer::Buffer: From<FixedSizePrimitiveArray<u8, false, Buffer>>,
{
    fn from(value: FixedSizeBinaryArray<N, false, Buffer>) -> Self {
        // todo(mbrobbel): const_assert
        assert!(N <= 0x7FFF_FFFF); // i32::MAX
        #[allow(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap
        )]
        arrow_array::FixedSizeBinaryArray::new(N as i32, value.0 .0.into(), None)
    }
}

impl<const NULLABLE: bool, const N: usize, Buffer: BufferType>
    From<FixedSizeBinaryArray<N, NULLABLE, Buffer>> for Arc<dyn arrow_array::Array>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    arrow_array::FixedSizeBinaryArray: From<FixedSizeBinaryArray<N, NULLABLE, Buffer>>,
{
    fn from(value: FixedSizeBinaryArray<N, NULLABLE, Buffer>) -> Self {
        Arc::new(arrow_array::FixedSizeBinaryArray::from(value))
    }
}

impl<const N: usize, Buffer: BufferType> From<FixedSizeBinaryArray<N, true, Buffer>>
    for arrow_array::FixedSizeBinaryArray
where
    arrow_buffer::Buffer: From<FixedSizePrimitiveArray<u8, false, Buffer>>,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn from(value: FixedSizeBinaryArray<N, true, Buffer>) -> Self {
        // todo(mbrobbel): const_assert
        assert!(N <= 0x7FFF_FFFF); // i32::MAX
        #[allow(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap
        )]
        arrow_array::FixedSizeBinaryArray::new(
            N as i32,
            value.0 .0.data.into(),
            Some(value.0 .0.validity.into()),
        )
    }
}

/// Panics when there are nulls
impl<const N: usize, Buffer: BufferType> From<arrow_array::FixedSizeBinaryArray>
    for FixedSizeBinaryArray<N, false, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: From<arrow_buffer::ScalarBuffer<u8>>,
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
    for FixedSizeBinaryArray<N, true, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: From<arrow_buffer::ScalarBuffer<u8>>,
    Bitmap<Buffer>: From<NullBuffer> + FromIterator<bool>,
{
    fn from(value: arrow_array::FixedSizeBinaryArray) -> Self {
        let (n, values, nulls_opt) = value.into_parts();
        assert_eq!(N, n.try_into().expect("size to cast to usize"));
        let data = arrow_buffer::ScalarBuffer::from(values).into();
        match nulls_opt {
            Some(null_buffer) => FixedSizeBinaryArray(FixedSizeListArray(Nullable {
                data,
                validity: null_buffer.into(),
            })),
            None => FixedSizeBinaryArray::<N, false, Buffer>(FixedSizeListArray(data)).into(),
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
            .collect::<FixedSizeBinaryArray<2, true>>();
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
        assert!(!FixedSizeBinaryArray::<2, true>::from(fixed_size_binary_array).any_null());
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let fixed_size_binary_array_nullable =
            arrow_array::FixedSizeBinaryArray::from(vec![None, Some([1_u8, 2, 3].as_slice())]);
        let _ = FixedSizeBinaryArray::<3, false>::from(fixed_size_binary_array_nullable);
    }

    #[test]
    fn into() {
        let fixed_size_binary_array =
            arrow_array::FixedSizeBinaryArray::try_from_iter(INPUT.into_iter()).expect("");
        assert_eq!(
            FixedSizeBinaryArray::<2, false>::from(fixed_size_binary_array)
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            INPUT.into_iter().flatten().collect::<Vec<_>>()
        );

        let fixed_size_binary_array_nullable_input = INPUT_NULLABLE
            .into_iter()
            .collect::<FixedSizeBinaryArray<2, true>>();
        let fixed_size_binary_array_nullable =
            arrow_array::FixedSizeBinaryArray::from(fixed_size_binary_array_nullable_input);
        assert_eq!(
            FixedSizeBinaryArray::<2, true>::from(fixed_size_binary_array_nullable)
                .into_iter()
                .collect::<Vec<_>>(),
            INPUT_NULLABLE
        );
    }
}
