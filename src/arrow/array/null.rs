//! Interop with `arrow-rs` boolean array.

use std::sync::Arc;

use crate::{
    array::{NullArray, Nulls, Unit},
    buffer::BufferType,
    validity::{Nullability, Validity},
    Length,
};
use arrow_array::Array;
use arrow_schema::{DataType, Field};

impl<T: Unit, const NULLABLE: bool, Buffer: BufferType> crate::arrow::Array
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

impl<T: Unit, Buffer: BufferType> From<NullArray<T, false, Buffer>> for Arc<dyn arrow_array::Array>
where
    arrow_array::NullArray: From<NullArray<T, false, Buffer>>,
{
    fn from(value: NullArray<T, false, Buffer>) -> Self {
        Arc::new(arrow_array::NullArray::from(value))
    }
}

impl<T: Unit, Buffer: BufferType> From<NullArray<T, false, Buffer>> for arrow_array::NullArray {
    fn from(value: NullArray<T, false, Buffer>) -> Self {
        arrow_array::NullArray::new(value.len())
    }
}

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
    #[cfg(feature = "derive")]
    fn derive() {
        use crate::array::StructArray;
        use arrow_array::cast::AsArray;
        use std::sync::Arc;

        #[derive(crate::ArrayType, Copy, Clone, Debug, Default)]
        struct Unit;

        #[derive(crate::ArrayType, Copy, Clone, Debug, Default)]
        struct NestedUnit(Unit);

        let input = [Unit; 4];
        let array = input.into_iter().collect::<NullArray<Unit>>();
        let arrow_array = arrow_array::NullArray::from(array);
        assert!(arrow_array.data_type().is_null());
        let narrow_array = NullArray::<Unit>::from(arrow_array);
        assert_eq!(narrow_array.len(), 4);

        let input_nested = [NestedUnit(Unit); 4];
        let array_nested = input_nested
            .into_iter()
            .collect::<StructArray<NestedUnit>>();
        let arrow_array_nested = arrow_array::StructArray::from(array_nested);
        assert!(arrow_array_nested
            .column(0)
            .as_struct()
            .column(0)
            .data_type()
            .is_null());
        assert_eq!(arrow_array_nested.len(), 4);
        let inner_unit = Arc::clone(arrow_array_nested.column(0).as_struct().column(0));
        let narrow_array_inner = NullArray::<Unit>::from(inner_unit);
        assert_eq!(narrow_array_inner.len(), 4);
        let narrow_array_nested = StructArray::<NestedUnit>::from(arrow_array_nested);
        assert_eq!(narrow_array_nested.len(), 4);
    }

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
            NullArray::<(), false, crate::arrow::buffer::ScalarBuffer>::from(null_array)
                .into_iter()
                .collect::<Vec<_>>(),
            INPUT
        );
    }
}
