//! Interop with `arrow-rs` fixed-sized primitive array.

use std::sync::Arc;

use arrow_array::types::{
    ArrowPrimitiveType, Float32Type, Float64Type, Int16Type, Int32Type, Int64Type, Int8Type,
    UInt16Type, UInt32Type, UInt64Type, UInt8Type,
};
use arrow_buffer::{NullBuffer, ScalarBuffer};
use arrow_schema::{DataType, Field};

use crate::{
    array::FixedSizePrimitiveArray, arrow::ArrowArray, bitmap::Bitmap, buffer::BufferType,
    nullable::Nullable, validity::Validity, FixedSize,
};

/// Create the `ArrowArray` impl and required conversions.
macro_rules! arrow_array_convert {
    ($ty:ty, $primitive_type:ident, $data_type:ident) => {
        impl<const NULLABLE: bool, Buffer: BufferType> ArrowArray
            for FixedSizePrimitiveArray<$ty, NULLABLE, Buffer>
        where
            <Buffer as BufferType>::Buffer<$ty>: Validity<NULLABLE>,
        {
            type Array = arrow_array::PrimitiveArray<$primitive_type>;

            fn as_field(name: &str) -> arrow_schema::Field {
                Field::new(name, DataType::$data_type, NULLABLE)
            }
        }

        impl<const NULLABLE: bool, Buffer: BufferType> From<Arc<dyn arrow_array::Array>>
            for FixedSizePrimitiveArray<$ty, NULLABLE, Buffer>
        where
            <Buffer as BufferType>::Buffer<$ty>: Validity<NULLABLE>,
            Self: From<arrow_array::PrimitiveArray<$primitive_type>>,
        {
            fn from(value: Arc<dyn arrow_array::Array>) -> Self {
                Self::from(arrow_array::PrimitiveArray::<$primitive_type>::from(
                    value.to_data(),
                ))
            }
        }
    };
}

arrow_array_convert!(u8, UInt8Type, UInt8);
arrow_array_convert!(u16, UInt16Type, UInt16);
arrow_array_convert!(u32, UInt32Type, UInt32);
arrow_array_convert!(u64, UInt64Type, UInt64);

arrow_array_convert!(i8, Int8Type, Int8);
arrow_array_convert!(i16, Int16Type, Int16);
arrow_array_convert!(i32, Int32Type, Int32);
arrow_array_convert!(i64, Int64Type, Int64);

arrow_array_convert!(f32, Float32Type, Float32);
arrow_array_convert!(f64, Float64Type, Float64);

impl<T: FixedSize, U: ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<FixedSizePrimitiveArray<T, false, Buffer>> for arrow_array::PrimitiveArray<U>
where
    <Buffer as BufferType>::Buffer<T>: Into<ScalarBuffer<T>>,
{
    fn from(value: FixedSizePrimitiveArray<T, false, Buffer>) -> Self {
        arrow_array::PrimitiveArray::new(value.0.into(), None)
    }
}

impl<T: FixedSize, U: ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<FixedSizePrimitiveArray<T, true, Buffer>> for arrow_array::PrimitiveArray<U>
where
    <Buffer as BufferType>::Buffer<T>: Into<ScalarBuffer<T>>,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn from(value: FixedSizePrimitiveArray<T, true, Buffer>) -> Self {
        arrow_array::PrimitiveArray::new(value.0.data.into(), Some(value.0.validity.into()))
    }
}

/// Panics when there are nulls
impl<T: FixedSize, U: ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<arrow_array::PrimitiveArray<U>> for FixedSizePrimitiveArray<T, false, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: From<ScalarBuffer<T>>,
{
    fn from(value: arrow_array::PrimitiveArray<U>) -> Self {
        let (_data_type, values, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => FixedSizePrimitiveArray(values.into()),
        }
    }
}

/// Panics when there are no nulls
impl<T: FixedSize, U: ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<arrow_array::PrimitiveArray<U>> for FixedSizePrimitiveArray<T, true, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: From<ScalarBuffer<T>>,
    Bitmap<Buffer>: From<NullBuffer>,
{
    fn from(value: arrow_array::PrimitiveArray<U>) -> Self {
        let (_data_type, values, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(null_buffer) => FixedSizePrimitiveArray(Nullable {
                data: values.into(),
                validity: null_buffer.into(),
            }),
            None => panic!("expected array with a null buffer"),
        }
    }
}

impl<Buffer: BufferType> From<arrow_buffer::Buffer> for FixedSizePrimitiveArray<u8, false, Buffer>
where
    <Buffer as BufferType>::Buffer<u8>: From<arrow_buffer::Buffer>,
{
    fn from(value: arrow_buffer::Buffer) -> Self {
        Self(value.into())
    }
}

#[cfg(test)]
mod tests {
    use arrow_array::types::{UInt16Type, UInt32Type};

    use crate::array::FixedSizePrimitiveArray;

    const INPUT: [u32; 4] = [1, 2, 3, 4];
    const INPUT_NULLABLE: [Option<u16>; 4] = [Some(1), None, Some(3), Some(4)];

    #[test]
    fn from() {
        let fixed_size_primitive_array = INPUT.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(
            arrow_array::PrimitiveArray::<UInt32Type>::from(fixed_size_primitive_array)
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            INPUT
        );

        let fixed_size_primitive_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<FixedSizePrimitiveArray<_, true>>();
        assert_eq!(
            arrow_array::PrimitiveArray::<UInt16Type>::from(fixed_size_primitive_array_nullable)
                .into_iter()
                .collect::<Vec<_>>(),
            INPUT_NULLABLE
        );
    }

    #[test]
    #[should_panic(expected = "expected array with a null buffer")]
    fn into_nullable() {
        let primitive_array = INPUT
            .into_iter()
            .collect::<arrow_array::PrimitiveArray<UInt32Type>>();
        let _ = FixedSizePrimitiveArray::<
            u32,
            true,
            crate::arrow::buffer::scalar_buffer::ArrowScalarBuffer,
        >::from(primitive_array);
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let primitive_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<arrow_array::PrimitiveArray<UInt16Type>>();
        let _ = FixedSizePrimitiveArray::<
            u16,
            false,
            crate::arrow::buffer::scalar_buffer::ArrowScalarBuffer,
        >::from(primitive_array_nullable);
    }

    #[test]
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn into() {
        let primitive_array = INPUT
            .into_iter()
            .collect::<arrow_array::PrimitiveArray<UInt32Type>>();
        assert_eq!(
            FixedSizePrimitiveArray::<
                u32,
                false,
                crate::arrow::buffer::scalar_buffer::ArrowScalarBuffer,
            >::from(primitive_array)
            .into_iter()
            .copied()
            .collect::<Vec<_>>(),
            INPUT
        );

        let primitive_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<arrow_array::PrimitiveArray<UInt16Type>>();
        assert_eq!(
            FixedSizePrimitiveArray::<
                u16,
                true,
                crate::arrow::buffer::scalar_buffer::ArrowScalarBuffer,
            >::from(primitive_array_nullable)
            .into_iter()
            .map(|opt| opt.copied())
            .collect::<Vec<_>>(),
            INPUT_NULLABLE
        );
    }
}
