//! Interop with `arrow-rs` fixed-sized primitive array.

use std::sync::Arc;

use arrow_array::{
    types::{
        ArrowPrimitiveType as _, Float32Type, Float64Type, Int16Type, Int32Type, Int64Type,
        Int8Type, UInt16Type, UInt32Type, UInt64Type, UInt8Type,
    },
    Array as _,
};
use arrow_buffer::ScalarBuffer;

use crate::{
    array::FixedSizePrimitiveArray,
    bitmap::Bitmap,
    buffer::{Buffer, BufferType},
    nullable::Nullable,
    validity::{Nullability, Validity},
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

impl<const NULLABLE: bool, T: FixedSize + FixedSizeExt, Buffer: BufferType> crate::arrow::Array
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    T: Nullability<NULLABLE>,
{
    type Array = arrow_array::PrimitiveArray<<T as FixedSizeExt>::ArrowPrimitiveType>;

    fn as_field(name: &str) -> arrow_schema::Field {
        arrow_schema::Field::new(
            name,
            <T as FixedSizeExt>::ArrowPrimitiveType::DATA_TYPE,
            NULLABLE,
        )
    }
}

impl<T: FixedSize, U: arrow_array::types::ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<FixedSizePrimitiveArray<T, false, Buffer>> for arrow_array::PrimitiveArray<U>
where
    arrow_buffer::ScalarBuffer<T>: From<<Buffer as BufferType>::Buffer<T>>,
{
    fn from(value: FixedSizePrimitiveArray<T, false, Buffer>) -> Self {
        arrow_array::PrimitiveArray::new(value.0.into(), None)
    }
}

impl<T: FixedSize, U: arrow_array::types::ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<FixedSizePrimitiveArray<T, true, Buffer>> for arrow_array::PrimitiveArray<U>
where
    arrow_buffer::ScalarBuffer<T>: From<<Buffer as BufferType>::Buffer<T>>,
    arrow_buffer::NullBuffer: From<Bitmap<Buffer>>,
{
    fn from(value: FixedSizePrimitiveArray<T, true, Buffer>) -> Self {
        arrow_array::PrimitiveArray::new(value.0.data.into(), Some(value.0.validity.into()))
    }
}

impl<T: FixedSizeExt, const NULLABLE: bool, Buffer: BufferType>
    From<FixedSizePrimitiveArray<T, NULLABLE, Buffer>> for Arc<dyn arrow_array::Array>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    arrow_array::PrimitiveArray<<T as FixedSizeExt>::ArrowPrimitiveType>:
        From<FixedSizePrimitiveArray<T, NULLABLE, Buffer>>,
{
    fn from(value: FixedSizePrimitiveArray<T, NULLABLE, Buffer>) -> Self {
        Arc::new(arrow_array::PrimitiveArray::from(value))
    }
}

/// Panics when there are nulls.
impl<T: FixedSize, U: arrow_array::types::ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<arrow_array::PrimitiveArray<U>> for FixedSizePrimitiveArray<T, false, Buffer>
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

/// Panics when there are no nulls.
impl<T: FixedSize, U: arrow_array::types::ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    From<arrow_array::PrimitiveArray<U>> for FixedSizePrimitiveArray<T, true, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: From<arrow_buffer::ScalarBuffer<T>>,
    Bitmap<Buffer>: From<arrow_buffer::NullBuffer>,
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

impl<const NULLABLE: bool, T: FixedSizeExt, Buffer: BufferType> From<Arc<dyn arrow_array::Array>>
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    Self: From<arrow_array::PrimitiveArray<<T as FixedSizeExt>::ArrowPrimitiveType>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::PrimitiveArray::from(value.to_data()))
    }
}

impl<T: FixedSize, Buffer: BufferType> From<arrow_buffer::ScalarBuffer<T>>
    for FixedSizePrimitiveArray<T, false, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: From<ScalarBuffer<T>>,
{
    fn from(value: arrow_buffer::ScalarBuffer<T>) -> Self {
        Self(value.into())
    }
}

impl<T: FixedSize, Buffer: BufferType> From<FixedSizePrimitiveArray<T, false, Buffer>>
    for ScalarBuffer<T>
where
    <Buffer as BufferType>::Buffer<T>: Into<ScalarBuffer<T>>,
{
    fn from(value: FixedSizePrimitiveArray<T, false, Buffer>) -> Self {
        value.0.into()
    }
}

impl<T: FixedSize, U: arrow_array::types::ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    PartialEq<arrow_array::PrimitiveArray<U>> for FixedSizePrimitiveArray<T, false, Buffer>
{
    fn eq(&self, other: &arrow_array::PrimitiveArray<U>) -> bool {
        other.nulls().is_none() && other.values().as_slice() == self.as_ref()
    }
}

impl<T: FixedSize, U: arrow_array::types::ArrowPrimitiveType<Native = T>, Buffer: BufferType>
    PartialEq<arrow_array::PrimitiveArray<U>> for FixedSizePrimitiveArray<T, true, Buffer>
where
    for<'a> &'a Self: IntoIterator<Item = Option<&'a T>>,
{
    fn eq(&self, other: &arrow_array::PrimitiveArray<U>) -> bool {
        self.len() == other.len() && self.into_iter().zip(other).all(|(a, b)| a.eq(&b.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use arrow_array::types::UInt32Type;

    use crate::{
        array::FixedSizePrimitiveArray,
        buffer::{BufferType, VecBuffer},
    };

    const INPUT: [u32; 4] = [1, 2, 3, 4];
    const INPUT_NULLABLE: [Option<u32>; 4] = [Some(1), None, Some(3), Some(4)];

    #[test]
    fn convert() {
        fn from<Buffer: BufferType>()
        where
            FixedSizePrimitiveArray<u32, false, Buffer>:
                Debug + FromIterator<u32> + Into<arrow_array::PrimitiveArray<UInt32Type>>,
            FixedSizePrimitiveArray<u32, true, Buffer>: Debug
                + FromIterator<Option<u32>>
                + Into<arrow_array::PrimitiveArray<UInt32Type>>
                + PartialEq<arrow_array::PrimitiveArray<UInt32Type>>,
        {
            let array_arrow: arrow_array::PrimitiveArray<UInt32Type> = INPUT
                .into_iter()
                .collect::<FixedSizePrimitiveArray<u32, false, Buffer>>()
                .into();
            let array_arrow_nullable: arrow_array::PrimitiveArray<UInt32Type> = INPUT_NULLABLE
                .into_iter()
                .collect::<FixedSizePrimitiveArray<u32, true, Buffer>>()
                .into();
            let array = INPUT
                .into_iter()
                .collect::<FixedSizePrimitiveArray<u32, false, Buffer>>();
            let array_nullable = INPUT_NULLABLE
                .into_iter()
                .collect::<FixedSizePrimitiveArray<u32, true, Buffer>>();
            assert_eq!(array, array_arrow);
            assert_eq!(array_nullable, array_arrow_nullable);
        }
        fn into<Buffer: BufferType>()
        where
            FixedSizePrimitiveArray<u32, false, Buffer>:
                From<arrow_array::PrimitiveArray<UInt32Type>> + Debug,
            FixedSizePrimitiveArray<u32, true, Buffer>: From<arrow_array::PrimitiveArray<UInt32Type>>
                + Debug
                + PartialEq<arrow_array::PrimitiveArray<UInt32Type>>,
        {
            let array_arrow = arrow_array::PrimitiveArray::<UInt32Type>::from(INPUT.to_vec());
            let array_arrow_nullable =
                arrow_array::PrimitiveArray::<UInt32Type>::from(INPUT_NULLABLE.to_vec());
            assert_eq!(
                FixedSizePrimitiveArray::<u32, false, Buffer>::from(array_arrow.clone()),
                array_arrow
            );
            assert_eq!(
                FixedSizePrimitiveArray::<u32, true, Buffer>::from(array_arrow_nullable.clone()),
                array_arrow_nullable
            );
        }

        from::<VecBuffer>();

        into::<VecBuffer>();
        into::<crate::arrow::buffer::ScalarBuffer>();
    }

    #[test]
    #[should_panic(expected = "expected array with a null buffer")]
    fn into_nullable() {
        let array = INPUT
            .into_iter()
            .collect::<arrow_array::PrimitiveArray<UInt32Type>>();
        let _ = FixedSizePrimitiveArray::<u32, true>::from(array);
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<arrow_array::PrimitiveArray<UInt32Type>>();
        let _ = FixedSizePrimitiveArray::<u32, false>::from(array_nullable);
    }
}
