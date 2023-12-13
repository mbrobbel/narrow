//! Interop with `arrow-rs` fixed-sized list array.

use std::sync::Arc;

use arrow_buffer::NullBuffer;
use arrow_schema::{DataType, Field};

use crate::{
    array::{Array, FixedSizeListArray},
    arrow::ArrowArray,
    bitmap::Bitmap,
    buffer::BufferType,
    nullable::Nullable,
    validity::Validity,
};

impl<const N: usize, T: ArrowArray, const NULLABLE: bool, Buffer: BufferType> ArrowArray
    for FixedSizeListArray<N, T, NULLABLE, Buffer>
where
    T: Validity<NULLABLE>,
{
    type Array = arrow_array::FixedSizeListArray;

    fn as_field(name: &str) -> Field {
        // todo(mbrobbel): const_assert
        assert!(N <= 0x7FFF_FFFF); // i32::MAX
        #[allow(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap
        )]
        Field::new(
            name,
            DataType::FixedSizeList(Arc::new(T::as_field("item")), N as i32),
            NULLABLE,
        )
    }
}

impl<const N: usize, T: Array, const NULLABLE: bool, Buffer: BufferType>
    From<Arc<dyn arrow_array::Array>> for FixedSizeListArray<N, T, NULLABLE, Buffer>
where
    T: Validity<NULLABLE>,
    Self: From<arrow_array::FixedSizeListArray>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::FixedSizeListArray::from(value.to_data()))
    }
}

impl<const N: usize, T: ArrowArray, Buffer: BufferType>
    From<FixedSizeListArray<N, T, false, Buffer>> for arrow_array::FixedSizeListArray
where
    <T as ArrowArray>::Array: From<T> + 'static,
{
    fn from(value: FixedSizeListArray<N, T, false, Buffer>) -> Self {
        // todo(mbrobbel): const_assert
        assert!(N <= 0x7FFF_FFFF); // i32::MAX
        #[allow(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap
        )]
        arrow_array::FixedSizeListArray::new(
            Arc::new(T::as_field("item")),
            N as i32,
            Arc::<<T as ArrowArray>::Array>::new(value.0.into()),
            None,
        )
    }
}

impl<const N: usize, T: ArrowArray, Buffer: BufferType> From<FixedSizeListArray<N, T, true, Buffer>>
    for arrow_array::FixedSizeListArray
where
    <T as ArrowArray>::Array: From<T> + 'static,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn from(value: FixedSizeListArray<N, T, true, Buffer>) -> Self {
        // todo(mbrobbel): const_assert
        assert!(N <= 0x7FFF_FFFF); // i32::MAX
        #[allow(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap
        )]
        arrow_array::FixedSizeListArray::new(
            Arc::new(T::as_field("item")),
            N as i32,
            Arc::<<T as ArrowArray>::Array>::new(value.0.data.into()),
            Some(value.0.validity.into()),
        )
    }
}

/// Panics when there are nulls
impl<const N: usize, T: ArrowArray, Buffer: BufferType> From<arrow_array::FixedSizeListArray>
    for FixedSizeListArray<N, T, false, Buffer>
where
    T: From<Arc<dyn arrow_array::Array>>,
{
    fn from(value: arrow_array::FixedSizeListArray) -> Self {
        let (_field, size, values, nulls_opt) = value.into_parts();
        let n = usize::try_from(size).expect("size to cast to usize");
        assert_eq!(N, n);
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => FixedSizeListArray(values.into()),
        }
    }
}

/// Panics when there are no nulls
impl<const N: usize, T: ArrowArray, Buffer: BufferType> From<arrow_array::FixedSizeListArray>
    for FixedSizeListArray<N, T, true, Buffer>
where
    T: From<Arc<dyn arrow_array::Array>>,
    Bitmap<Buffer>: From<NullBuffer>,
{
    fn from(value: arrow_array::FixedSizeListArray) -> Self {
        let (_field, size, values, nulls_opt) = value.into_parts();
        let n = usize::try_from(size).expect("size to cast to usize");
        assert_eq!(N, n);
        match nulls_opt {
            Some(null_buffer) => FixedSizeListArray(Nullable {
                data: values.into(),
                validity: null_buffer.into(),
            }),
            None => panic!("expected array with a null buffer"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        array::{StringArray, Uint32Array},
        arrow::scalar_buffer::ArrowScalarBuffer,
    };

    use super::*;

    const INPUT: [[u32; 2]; 3] = [[1, 2], [3, 4], [5, 6]];
    const INPUT_NULLABLE: [Option<[&str; 2]>; 3] =
        [Some(["hello", " "]), None, Some(["world", "!"])];

    #[test]
    fn from() {
        let fixed_size_list_array = INPUT
            .into_iter()
            .collect::<FixedSizeListArray<2, Uint32Array>>();
        assert_eq!(
            arrow_array::FixedSizeListArray::from(fixed_size_list_array)
                .iter()
                .flatten()
                .flat_map(|dyn_array| {
                    let array: Uint32Array<false, ArrowScalarBuffer> = dyn_array.into();
                    array.into_iter().copied().collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            INPUT.into_iter().flatten().collect::<Vec<_>>()
        );

        let fixed_size_list_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<FixedSizeListArray<2, StringArray, true>>();
        assert_eq!(
            arrow_array::FixedSizeListArray::from(fixed_size_list_array_nullable)
                .iter()
                .flatten()
                .flat_map(|dyn_array| {
                    let array: StringArray<false, i32, ArrowScalarBuffer> = dyn_array.into();
                    array.into_iter().map(ToOwned::to_owned).collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            INPUT_NULLABLE
                .into_iter()
                .flatten()
                .flatten()
                .collect::<Vec<_>>()
        );
    }

    // #[test]
    // #[should_panic(expected = "expected array with a null buffer")]
    // fn into_nullable() {
    //     let primitive_array = INPUT
    //         .into_iter()
    //         .collect::<arrow_array::PrimitiveArray<UInt32Type>>();
    //     let _ = FixedSizePrimitiveArray::<
    //         u32,
    //         true,
    //         crate::arrow::buffer::scalar_buffer::ArrowScalarBuffer,
    //     >::from(primitive_array);
    // }

    // #[test]
    // #[should_panic(expected = "expected array without a null buffer")]
    // fn into_non_nullable() {
    //     let primitive_array_nullable = INPUT_NULLABLE
    //         .into_iter()
    //         .collect::<arrow_array::PrimitiveArray<UInt16Type>>();
    //     let _ = FixedSizePrimitiveArray::<
    //         u16,
    //         false,
    //         crate::arrow::buffer::scalar_buffer::ArrowScalarBuffer,
    //     >::from(primitive_array_nullable);
    // }

    #[test]
    fn into() {
        // let primitive_array = INPUT
        //     .into_iter()
        //     .collect::<arrow_array::PrimitiveArray<UInt32Type>>();
        // assert_eq!(
        //     FixedSizePrimitiveArray::<
        //         u32,
        //         false,
        //         crate::arrow::buffer::scalar_buffer::ArrowScalarBuffer,
        //     >::from(primitive_array)
        //     .into_iter()
        //     .copied()
        //     .collect::<Vec<_>>(),
        //     INPUT
        // );

        // let primitive_array_nullable = INPUT_NULLABLE
        //     .into_iter()
        //     .collect::<arrow_array::PrimitiveArray<UInt16Type>>();
        // assert_eq!(
        //     FixedSizePrimitiveArray::<
        //         u16,
        //         true,
        //         crate::arrow::buffer::scalar_buffer::ArrowScalarBuffer,
        //     >::from(primitive_array_nullable)
        //     .into_iter()
        //     .map(|opt| opt.copied())
        //     .collect::<Vec<_>>(),
        //     INPUT_NULLABLE
        // );
    }
}
