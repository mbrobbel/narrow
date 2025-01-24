//! Interop with `arrow-rs` fixed-sized list array.

use std::sync::Arc;

use arrow_buffer::NullBuffer;
use arrow_schema::{DataType, Field};

use crate::{
    array::{Array, FixedSizeListArray},
    bitmap::Bitmap,
    buffer::BufferType,
    nullability::{NonNullable, Nullability, Nullable},
    validity::Validity,
    Length,
};

impl<const N: usize, T: crate::arrow::Array, Nullable: Nullability, Buffer: BufferType>
    crate::arrow::Array for FixedSizeListArray<N, T, Nullable, Buffer>
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
        Field::new(name, Self::data_type(), Nullable::NULLABLE)
    }

    fn data_type() -> arrow_schema::DataType {
        DataType::FixedSizeList(
            Arc::new(T::as_field("item")),
            i32::try_from(N).expect("overflow"),
        )
    }
}

impl<const N: usize, T: Array, Nullable: Nullability, Buffer: BufferType>
    From<Arc<dyn arrow_array::Array>> for FixedSizeListArray<N, T, Nullable, Buffer>
where
    Self: From<arrow_array::FixedSizeListArray>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::FixedSizeListArray::from(value.to_data()))
    }
}

impl<const N: usize, T: Array, Buffer: BufferType>
    From<FixedSizeListArray<N, T, NonNullable, Buffer>> for Arc<dyn arrow_array::Array>
where
    T: crate::arrow::Array + Into<Arc<dyn arrow_array::Array>>,
{
    fn from(value: FixedSizeListArray<N, T, NonNullable, Buffer>) -> Self {
        Arc::new(arrow_array::FixedSizeListArray::new(
            Arc::new(T::as_field("item")),
            i32::try_from(N).expect("overflow"),
            value.0.into(),
            None,
        ))
    }
}

impl<const N: usize, T: Array, Buffer: BufferType> From<FixedSizeListArray<N, T, Nullable, Buffer>>
    for Arc<dyn arrow_array::Array>
where
    T: crate::arrow::Array + Into<Arc<dyn arrow_array::Array>>,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn from(value: FixedSizeListArray<N, T, Nullable, Buffer>) -> Self {
        Arc::new(arrow_array::FixedSizeListArray::new(
            Arc::new(T::as_field("item")),
            i32::try_from(N).expect("overflow"),
            value.0.data.into(),
            Some(value.0.validity.into()),
        ))
    }
}

impl<const N: usize, T: crate::arrow::Array, Buffer: BufferType>
    From<FixedSizeListArray<N, T, NonNullable, Buffer>> for arrow_array::FixedSizeListArray
where
    <T as crate::arrow::Array>::Array: From<T> + 'static,
{
    fn from(value: FixedSizeListArray<N, T, NonNullable, Buffer>) -> Self {
        // todo(mbrobbel): const_assert
        assert!(N <= 0x7FFF_FFFF); // i32::MAX
        #[allow(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap
        )]
        arrow_array::FixedSizeListArray::new(
            Arc::new(T::as_field("item")),
            i32::try_from(N).expect("overflow"),
            Arc::<<T as crate::arrow::Array>::Array>::new(value.0.into()),
            None,
        )
    }
}

impl<const N: usize, T: crate::arrow::Array, Buffer: BufferType>
    From<FixedSizeListArray<N, T, Nullable, Buffer>> for arrow_array::FixedSizeListArray
where
    <T as crate::arrow::Array>::Array: From<T> + 'static,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn from(value: FixedSizeListArray<N, T, Nullable, Buffer>) -> Self {
        // todo(mbrobbel): const_assert
        assert!(N <= 0x7FFF_FFFF); // i32::MAX
        #[allow(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap
        )]
        arrow_array::FixedSizeListArray::new(
            Arc::new(T::as_field("item")),
            i32::try_from(N).expect("overflow"),
            Arc::<<T as crate::arrow::Array>::Array>::new(value.0.data.into()),
            Some(value.0.validity.into()),
        )
    }
}

/// Panics when there are nulls
impl<const N: usize, T: crate::arrow::Array, Buffer: BufferType>
    From<arrow_array::FixedSizeListArray> for FixedSizeListArray<N, T, NonNullable, Buffer>
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

impl<const N: usize, T: crate::arrow::Array, Buffer: BufferType>
    From<arrow_array::FixedSizeListArray> for FixedSizeListArray<N, T, Nullable, Buffer>
where
    T: From<Arc<dyn arrow_array::Array>> + Length,
    Bitmap<Buffer>: From<NullBuffer> + FromIterator<bool>,
{
    fn from(value: arrow_array::FixedSizeListArray) -> Self {
        let (_field, size, values, nulls_opt) = value.into_parts();
        let n = usize::try_from(size).expect("size to cast to usize");
        assert_eq!(N, n);
        let data = values.into();
        match nulls_opt {
            Some(null_buffer) => FixedSizeListArray(Validity {
                data,
                validity: null_buffer.into(),
            }),
            None => FixedSizeListArray::<N, T, NonNullable, Buffer>(data).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use arrow_array::types::UInt32Type;

    use crate::{
        array::{StringArray, Uint32Array},
        bitmap::ValidityBitmap,
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
                    let array: Uint32Array<NonNullable, crate::arrow::buffer::ScalarBuffer> =
                        dyn_array.into();
                    array.into_iter().copied().collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            INPUT.into_iter().flatten().collect::<Vec<_>>()
        );

        let fixed_size_list_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<FixedSizeListArray<2, StringArray, Nullable>>();
        assert_eq!(
            arrow_array::FixedSizeListArray::from(fixed_size_list_array_nullable)
                .iter()
                .flatten()
                .flat_map(|dyn_array| {
                    StringArray::<NonNullable, i32, crate::arrow::buffer::ScalarBuffer>::from(
                        dyn_array,
                    )
                    .into_iter()
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
                })
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
        let fixed_size_list_array =
            arrow_array::FixedSizeListArray::from_iter_primitive::<UInt32Type, _, _>(
                INPUT
                    .into_iter()
                    .map(|opt| opt.into_iter().map(Option::Some))
                    .map(Option::Some),
                2,
            );
        assert!(!FixedSizeListArray::<
            2,
            Uint32Array<NonNullable, crate::arrow::buffer::ScalarBuffer>,
            Nullable,
            crate::arrow::buffer::ScalarBuffer,
        >::from(fixed_size_list_array)
        .any_null());
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let fixed_size_list_array_nullable =
            arrow_array::FixedSizeListArray::from_iter_primitive::<UInt32Type, _, _>(
                vec![Some(vec![Some(0), Some(1), Some(2)]), None],
                3,
            );
        let _ = FixedSizeListArray::<
            3,
            Uint32Array<NonNullable, crate::arrow::buffer::ScalarBuffer>,
            NonNullable,
            crate::arrow::buffer::ScalarBuffer,
        >::from(fixed_size_list_array_nullable);
    }

    #[test]
    fn into() {
        let fixed_size_list_array =
            arrow_array::FixedSizeListArray::from_iter_primitive::<UInt32Type, _, _>(
                INPUT
                    .into_iter()
                    .map(|opt| opt.into_iter().map(Option::Some))
                    .map(Option::Some),
                2,
            );
        assert_eq!(
            FixedSizeListArray::<
                2,
                Uint32Array<NonNullable, crate::arrow::buffer::ScalarBuffer>,
                NonNullable,
                crate::arrow::buffer::ScalarBuffer,
            >::from(fixed_size_list_array)
            .into_iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>(),
            INPUT.into_iter().flatten().collect::<Vec<_>>()
        );

        let fixed_size_list_array_nullable_input = INPUT_NULLABLE
            .into_iter()
            .collect::<FixedSizeListArray<2, StringArray, Nullable>>();
        let fixed_size_list_array_nullable =
            arrow_array::FixedSizeListArray::from(fixed_size_list_array_nullable_input);

        assert_eq!(
            FixedSizeListArray::<
                2,
                StringArray<NonNullable, i32, crate::arrow::buffer::ScalarBuffer>,
                Nullable,
            >::from(fixed_size_list_array_nullable)
            .into_iter()
            .collect::<Vec<_>>(),
            INPUT_NULLABLE
        );
    }
}
