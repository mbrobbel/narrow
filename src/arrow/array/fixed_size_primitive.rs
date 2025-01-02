//! Interop with `arrow-rs` fixed-sized primitive array.

use std::sync::Arc;

use arrow_array::{
    types::{
        ArrowPrimitiveType as _, Float32Type, Float64Type, Int16Type, Int32Type, Int64Type,
        Int8Type, UInt16Type, UInt32Type, UInt64Type, UInt8Type,
    },
    Array as _,
};

use crate::{
    array::FixedSizePrimitiveArray,
    bitmap::Bitmap,
    buffer::{Buffer, BufferType},
    nullability::{NonNullable, Nullability, Nullable},
    validity::Validity,
    FixedSize, Length,
};

/// Mapping between [`FixedSize`] types and [`arrow_array::types::ArrowPrimitiveType`].
pub trait FixedSizeExt: FixedSize {
    /// The corresponding [`arrow_array::types::ArrowPrimitiveType`] for a [`FixedSize`] type.
    type ArrowPrimitiveType: arrow_array::types::ArrowPrimitiveType;
}

impl FixedSizeExt for u8 {
    type ArrowPrimitiveType = UInt8Type;
}
impl FixedSizeExt for u16 {
    type ArrowPrimitiveType = UInt16Type;
}
impl FixedSizeExt for u32 {
    type ArrowPrimitiveType = UInt32Type;
}
impl FixedSizeExt for u64 {
    type ArrowPrimitiveType = UInt64Type;
}
impl FixedSizeExt for i8 {
    type ArrowPrimitiveType = Int8Type;
}
impl FixedSizeExt for i16 {
    type ArrowPrimitiveType = Int16Type;
}
impl FixedSizeExt for i32 {
    type ArrowPrimitiveType = Int32Type;
}
impl FixedSizeExt for i64 {
    type ArrowPrimitiveType = Int64Type;
}
impl FixedSizeExt for f32 {
    type ArrowPrimitiveType = Float32Type;
}
impl FixedSizeExt for f64 {
    type ArrowPrimitiveType = Float64Type;
}

impl<Nullable: Nullability, T: FixedSize + FixedSizeExt, Buffer: BufferType> crate::arrow::Array
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
{
    type Array = arrow_array::PrimitiveArray<<T as FixedSizeExt>::ArrowPrimitiveType>;

    fn as_field(name: &str) -> arrow_schema::Field {
        arrow_schema::Field::new(name, Self::data_type(), Nullable::NULLABLE)
    }

    fn data_type() -> arrow_schema::DataType {
        <T as FixedSizeExt>::ArrowPrimitiveType::DATA_TYPE
    }
}

impl<T: FixedSize, U: arrow_array::types::ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    Into<arrow_array::PrimitiveArray<U>> for FixedSizePrimitiveArray<T, NonNullable, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Into<arrow_buffer::ScalarBuffer<T>>,
{
    fn into(self) -> arrow_array::PrimitiveArray<U> {
        arrow_array::PrimitiveArray::new(self.0.into(), None)
    }
}

impl<T: FixedSize, U: arrow_array::types::ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    Into<arrow_array::PrimitiveArray<U>> for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Into<arrow_buffer::ScalarBuffer<T>>,
    Bitmap<Buffer>: Into<arrow_buffer::NullBuffer>,
{
    fn into(self) -> arrow_array::PrimitiveArray<U> {
        arrow_array::PrimitiveArray::new(self.0.data.into(), Some(self.0.validity.into()))
    }
}

impl<T: FixedSizeExt, Nullable: Nullability, Buffer: BufferType> Into<Arc<dyn arrow_array::Array>>
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    FixedSizePrimitiveArray<T, Nullable, Buffer>:
        Into<arrow_array::PrimitiveArray<<T as FixedSizeExt>::ArrowPrimitiveType>>,
{
    fn into(self) -> Arc<dyn arrow_array::Array> {
        Arc::new(self.into())
    }
}

/// Panics when there are nulls.
impl<T: FixedSize, U: arrow_array::types::ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<arrow_array::PrimitiveArray<U>> for FixedSizePrimitiveArray<T, NonNullable, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: From<arrow_buffer::ScalarBuffer<T>>,
{
    fn from(value: arrow_array::PrimitiveArray<U>) -> Self {
        let (_data_type, values, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => FixedSizePrimitiveArray(values.into()),
        }
    }
}

impl<T: FixedSize, U: arrow_array::types::ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<arrow_array::PrimitiveArray<U>> for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: From<arrow_buffer::ScalarBuffer<T>>,
    Bitmap<Buffer>: From<arrow_buffer::NullBuffer> + FromIterator<bool>,
{
    fn from(value: arrow_array::PrimitiveArray<U>) -> Self {
        let (_data_type, values, nulls_opt) = value.into_parts();
        let data = values.into();
        match nulls_opt {
            Some(null_buffer) => FixedSizePrimitiveArray(Validity {
                data,
                validity: null_buffer.into(),
            }),
            None => FixedSizePrimitiveArray::<T, NonNullable, Buffer>(data).into(),
        }
    }
}

impl<Nullable: Nullability, T: FixedSizeExt, Buffer: BufferType> From<Arc<dyn arrow_array::Array>>
    for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    Self: From<arrow_array::PrimitiveArray<<T as FixedSizeExt>::ArrowPrimitiveType>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::PrimitiveArray::from(value.to_data()))
    }
}

impl<T: FixedSize, Buffer: BufferType> From<arrow_buffer::ScalarBuffer<T>>
    for FixedSizePrimitiveArray<T, NonNullable, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: From<arrow_buffer::ScalarBuffer<T>>,
{
    fn from(value: arrow_buffer::ScalarBuffer<T>) -> Self {
        Self(value.into())
    }
}

impl<T: FixedSize, Buffer: BufferType> Into<arrow_buffer::ScalarBuffer<T>>
    for FixedSizePrimitiveArray<T, NonNullable, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Into<arrow_buffer::ScalarBuffer<T>>,
{
    fn into(self) -> arrow_buffer::ScalarBuffer<T> {
        self.0.into()
    }
}

impl<T: FixedSize, Buffer: BufferType> Into<arrow_buffer::Buffer>
    for FixedSizePrimitiveArray<T, NonNullable, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Into<arrow_buffer::Buffer>,
{
    fn into(self) -> arrow_buffer::Buffer {
        self.0.into()
    }
}

impl<T: FixedSize, U: arrow_array::types::ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    PartialEq<arrow_array::PrimitiveArray<U>> for FixedSizePrimitiveArray<T, NonNullable, Buffer>
{
    fn eq(&self, other: &arrow_array::PrimitiveArray<U>) -> bool {
        other.nulls().is_none() && other.values().as_slice() == self.as_ref()
    }
}

impl<T: FixedSize, U: arrow_array::types::ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    PartialEq<arrow_array::PrimitiveArray<U>> for FixedSizePrimitiveArray<T, Nullable, Buffer>
where
    for<'a> &'a Self: IntoIterator<Item = Option<&'a T>>,
{
    fn eq(&self, other: &arrow_array::PrimitiveArray<U>) -> bool {
        self.len() == other.len() && self.into_iter().zip(other).all(|(a, b)| a.eq(&b.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fmt::Debug;

    use arrow_array::types::UInt32Type;

    use crate::{
        array::FixedSizePrimitiveArray,
        bitmap::ValidityBitmap,
        buffer::{BufferType, VecBuffer},
    };

    const INPUT: [u32; 4] = [1, 2, 3, 4];
    const INPUT_NULLABLE: [Option<u32>; 4] = [Some(1), None, Some(3), Some(4)];

    #[test]
    fn convert() {
        fn from<Buffer: BufferType>()
        where
            FixedSizePrimitiveArray<u32, NonNullable, Buffer>:
                Debug + FromIterator<u32> + Into<arrow_array::PrimitiveArray<UInt32Type>>,
            FixedSizePrimitiveArray<u32, Nullable, Buffer>: Debug
                + FromIterator<Option<u32>>
                + Into<arrow_array::PrimitiveArray<UInt32Type>>
                + PartialEq<arrow_array::PrimitiveArray<UInt32Type>>,
        {
            let array_arrow: arrow_array::PrimitiveArray<UInt32Type> = INPUT
                .into_iter()
                .collect::<FixedSizePrimitiveArray<u32, NonNullable, Buffer>>()
                .into();
            let array_arrow_nullable: arrow_array::PrimitiveArray<UInt32Type> = INPUT_NULLABLE
                .into_iter()
                .collect::<FixedSizePrimitiveArray<u32, Nullable, Buffer>>()
                .into();
            let array = INPUT
                .into_iter()
                .collect::<FixedSizePrimitiveArray<u32, NonNullable, Buffer>>();
            let array_nullable = INPUT_NULLABLE
                .into_iter()
                .collect::<FixedSizePrimitiveArray<u32, Nullable, Buffer>>();
            assert_eq!(array, array_arrow);
            assert_eq!(array_nullable, array_arrow_nullable);
        }
        fn into<Buffer: BufferType>()
        where
            FixedSizePrimitiveArray<u32, NonNullable, Buffer>:
                From<arrow_array::PrimitiveArray<UInt32Type>> + Debug,
            FixedSizePrimitiveArray<u32, Nullable, Buffer>: From<arrow_array::PrimitiveArray<UInt32Type>>
                + Debug
                + PartialEq<arrow_array::PrimitiveArray<UInt32Type>>,
        {
            let array_arrow = arrow_array::PrimitiveArray::<UInt32Type>::from(INPUT.to_vec());
            let array_arrow_nullable =
                arrow_array::PrimitiveArray::<UInt32Type>::from(INPUT_NULLABLE.to_vec());
            assert_eq!(
                FixedSizePrimitiveArray::<u32, NonNullable, Buffer>::from(array_arrow.clone()),
                array_arrow
            );
            assert_eq!(
                FixedSizePrimitiveArray::<u32, Nullable, Buffer>::from(
                    array_arrow_nullable.clone()
                ),
                array_arrow_nullable
            );
        }

        from::<VecBuffer>();

        into::<VecBuffer>();
        into::<crate::arrow::buffer::ScalarBuffer>();
    }

    #[test]
    fn into_nullable() {
        let array = INPUT
            .into_iter()
            .collect::<arrow_array::PrimitiveArray<UInt32Type>>();
        assert!(!FixedSizePrimitiveArray::<u32, Nullable>::from(array).any_null());
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<arrow_array::PrimitiveArray<UInt32Type>>();
        let _ = FixedSizePrimitiveArray::<u32, NonNullable>::from(array_nullable);
    }
}
