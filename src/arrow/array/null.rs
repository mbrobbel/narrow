//! Interop with `arrow-rs` boolean array.

use std::sync::Arc;

use crate::{
    array::{NullArray, Nulls, Unit},
    arrow::ArrowArray,
    buffer::BufferType,
    validity::{Nullability, Validity},
    Length,
};
use arrow_array::Array;
use arrow_schema::{DataType, Field};

impl<T: Unit, const NULLABLE: bool, Buffer: BufferType> ArrowArray
    for NullArray<T, NULLABLE, Buffer>
where
    T: Nullability<NULLABLE>,
    Nulls<T>: Validity<NULLABLE>,
{
    type Array = arrow_array::NullArray;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(name, DataType::Null, NULLABLE)
    }
}

impl<T: Unit, Buffer: BufferType> From<Arc<dyn arrow_array::Array>> for NullArray<T, false, Buffer>
where
    Self: From<arrow_array::NullArray>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::NullArray::from(value.to_data()))
    }
}

impl<T: Unit, Buffer: BufferType> From<NullArray<T, false, Buffer>> for arrow_array::NullArray {
    fn from(value: NullArray<T, false, Buffer>) -> Self {
        arrow_array::NullArray::new(value.len())
    }
}

/// Panics when there are nulls
impl<T: Unit, Buffer: BufferType> From<arrow_array::NullArray> for NullArray<T, false, Buffer> {
    fn from(value: arrow_array::NullArray) -> Self {
        NullArray(Nulls::new(value.len()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{array::NullArray, buffer::ArcBuffer, Length};
    use arrow_array::Array;

    const INPUT: [(); 4] = [(), (), (), ()];

    #[test]
    fn from() {
        let null_array = INPUT.into_iter().collect::<NullArray>();
        assert_eq!(
            arrow_array::NullArray::new(null_array.len()).len(),
            INPUT.len()
        );

        let null_array_arc = INPUT
            .into_iter()
            .collect::<NullArray<_, false, ArcBuffer>>();
        assert_eq!(
            arrow_array::NullArray::new(null_array_arc.len()).len(),
            INPUT.len()
        );
    }

    #[test]
    fn into() {
        let null_array = arrow_array::NullArray::new(INPUT.len());
        assert_eq!(
            NullArray::<(), false, crate::arrow::buffer::scalar_buffer::ArrowScalarBuffer>::from(
                null_array
            )
            .into_iter()
            .collect::<Vec<_>>(),
            INPUT
        );
    }
}
