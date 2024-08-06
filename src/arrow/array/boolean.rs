//! Interop with `arrow-rs` boolean array.

use std::sync::Arc;

use arrow_array::Array as _;

use crate::{
    array::BooleanArray,
    bitmap::Bitmap,
    buffer::BufferType,
    nullable::Nullable,
    validity::{Nullability, Validity},
    Length,
};

impl<const NULLABLE: bool, Buffer: BufferType> crate::arrow::Array
    for BooleanArray<NULLABLE, Buffer>
where
    Bitmap<Buffer>: Validity<NULLABLE>,
    bool: Nullability<NULLABLE>,
{
    type Array = arrow_array::BooleanArray;

    fn as_field(name: &str) -> arrow_schema::Field {
        arrow_schema::Field::new(name, arrow_schema::DataType::Boolean, NULLABLE)
    }
}

impl<Buffer: BufferType> From<BooleanArray<false, Buffer>> for arrow_array::BooleanArray
where
    arrow_buffer::BooleanBuffer: From<Bitmap<Buffer>>,
{
    fn from(value: BooleanArray<false, Buffer>) -> Self {
        arrow_array::BooleanArray::new(value.0.into(), None)
    }
}

impl<Buffer: BufferType> From<BooleanArray<true, Buffer>> for arrow_array::BooleanArray
where
    arrow_buffer::BooleanBuffer: From<Bitmap<Buffer>>,
    arrow_buffer::NullBuffer: From<Bitmap<Buffer>>,
{
    fn from(value: BooleanArray<true, Buffer>) -> Self {
        arrow_array::BooleanArray::new(value.0.data.into(), Some(value.0.validity.into()))
    }
}

impl<const NULLABLE: bool, Buffer: BufferType> From<BooleanArray<NULLABLE, Buffer>>
    for Arc<dyn arrow_array::Array>
where
    Bitmap<Buffer>: Validity<NULLABLE>,
    arrow_array::BooleanArray: From<BooleanArray<NULLABLE, Buffer>>,
{
    fn from(value: BooleanArray<NULLABLE, Buffer>) -> Self {
        Arc::new(arrow_array::BooleanArray::from(value))
    }
}

/// Panics when there are nulls
impl<Buffer: BufferType> From<arrow_array::BooleanArray> for BooleanArray<false, Buffer>
where
    Bitmap<Buffer>: From<arrow_buffer::BooleanBuffer>,
{
    fn from(value: arrow_array::BooleanArray) -> Self {
        let (boolean_buffer, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => BooleanArray(boolean_buffer.into()),
        }
    }
}

impl<Buffer: BufferType> From<arrow_array::BooleanArray> for BooleanArray<true, Buffer>
where
    Bitmap<Buffer>:
        From<arrow_buffer::BooleanBuffer> + From<arrow_buffer::NullBuffer> + FromIterator<bool>,
{
    fn from(value: arrow_array::BooleanArray) -> Self {
        let (boolean_buffer, nulls_opt) = value.into_parts();
        let data = boolean_buffer.into();
        match nulls_opt {
            Some(null_buffer) => BooleanArray(Nullable {
                data,
                validity: null_buffer.into(),
            }),
            None => BooleanArray::<false, Buffer>(data).into(),
        }
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

impl<Buffer: BufferType> PartialEq<arrow_array::BooleanArray> for BooleanArray<false, Buffer> {
    fn eq(&self, other: &arrow_array::BooleanArray) -> bool {
        other.nulls().is_none()
            && self.len() == other.len()
            && other.values().iter().zip(self).all(|(a, b)| a == b)
    }
}

impl<Buffer: BufferType> PartialEq<arrow_array::BooleanArray> for BooleanArray<true, Buffer> {
    fn eq(&self, other: &arrow_array::BooleanArray) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(a, b)| a == b)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        array::BooleanArray,
        bitmap::ValidityBitmap,
        buffer::{BufferType, VecBuffer},
    };

    const INPUT: [bool; 4] = [true, true, false, true];
    const INPUT_NULLABLE: [Option<bool>; 4] = [Some(true), None, Some(false), Some(true)];

    #[test]
    fn convert() {
        fn from<Buffer: BufferType>()
        where
            BooleanArray<false, Buffer>: FromIterator<bool> + Into<arrow_array::BooleanArray>,
            BooleanArray<true, Buffer>:
                FromIterator<Option<bool>> + Into<arrow_array::BooleanArray>,
        {
            let array_arrow: arrow_array::BooleanArray = INPUT
                .into_iter()
                .collect::<BooleanArray<false, Buffer>>()
                .into();
            let array_arrow_nullable: arrow_array::BooleanArray = INPUT_NULLABLE
                .into_iter()
                .collect::<BooleanArray<true, Buffer>>()
                .into();
            let array = INPUT.into_iter().collect::<BooleanArray<false, Buffer>>();
            let array_nullable = INPUT_NULLABLE
                .into_iter()
                .collect::<BooleanArray<true, Buffer>>();
            assert_eq!(array, array_arrow);
            assert_eq!(array_nullable, array_arrow_nullable);
        }
        fn into<Buffer: BufferType>()
        where
            BooleanArray<false, Buffer>: From<arrow_array::BooleanArray>,
            BooleanArray<true, Buffer>: From<arrow_array::BooleanArray>,
        {
            let array_arrow = arrow_array::BooleanArray::from(INPUT.to_vec());
            let array_arrow_nullable = arrow_array::BooleanArray::from(INPUT_NULLABLE.to_vec());
            assert_eq!(
                BooleanArray::<false, Buffer>::from(array_arrow.clone()),
                array_arrow
            );
            assert_eq!(
                BooleanArray::<true, Buffer>::from(array_arrow_nullable.clone()),
                array_arrow_nullable
            );
        }

        from::<VecBuffer>();
        // from::<ArcBuffer>(); missing Extend for Arc<[u8]>
        // from::<BoxBuffer>(); missing Extend for Box<[u8]>
        // from::<crate::arrow::buffer::ScalarBuffer>(); is not BufferMut
        from::<crate::arrow::buffer::BufferBuilder>();

        into::<VecBuffer>();
        // into::<ArcBuffer>(); missing ScalarBuffer<u8> from Arc<[u8]>
        // into::<BoxBuffer>(); missing ScalarBuffer<u8> from Box<[u8]>
        into::<crate::arrow::buffer::ScalarBuffer>();
        // into::<crate::arrow::buffer::BufferBuilder>(); missing BufferBuilder<u8> from ScalarBuffer<u8>
    }

    #[test]
    fn into_nullable() {
        let array = arrow_array::BooleanArray::from(INPUT.to_vec());
        assert!(!BooleanArray::<true>::from(array).any_null());
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let array_nullable = arrow_array::BooleanArray::from(INPUT_NULLABLE.to_vec());
        let _ = BooleanArray::<false>::from(array_nullable);
    }
}
