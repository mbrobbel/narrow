//! Interop with `arrow-rs` boolean array.

use std::sync::Arc;

use crate::{
    array::{NullArray, Nulls, Unit},
    buffer::BufferType,
    nullability::{NonNullable, Nullability},
    Length,
};
use arrow_array::Array;
use arrow_schema::{DataType, Field};

impl<T: Unit, Nullable: Nullability, Buffer: BufferType> crate::arrow::Array
    for NullArray<T, Nullable, Buffer>
{
    type Array = arrow_array::NullArray;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(name, Self::data_type(), Nullable::NULLABLE)
    }

    fn data_type() -> arrow_schema::DataType {
        DataType::Null
    }
}

impl<T: Unit, Buffer: BufferType> From<Arc<dyn arrow_array::Array>>
    for NullArray<T, NonNullable, Buffer>
where
    Self: From<arrow_array::NullArray>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::NullArray::from(value.to_data()))
    }
}


impl<T: Unit, Buffer: BufferType> Into<Arc<dyn arrow_array::Array>>
    for NullArray<T, NonNullable, Buffer>
where
    NullArray<T, NonNullable, Buffer>: Into<arrow_array::NullArray>,
{
    fn into(self) -> Arc<dyn arrow_array::Array> {
        Arc::new(self.into())
    }
}


impl<T: Unit, Buffer: BufferType> Into<arrow_array::NullArray>
    for NullArray<T, NonNullable, Buffer>
{
    fn into(self) -> arrow_array::NullArray {
        arrow_array::NullArray::new(self.len())
    }
}

impl<T: Unit, Buffer: BufferType> From<arrow_array::NullArray>
    for NullArray<T, NonNullable, Buffer>
{
    fn from(value: arrow_array::NullArray) -> Self {
        NullArray(Nulls::new(value.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::buffer::ArcBuffer;

    const INPUT: [(); 4] = [(), (), (), ()];

    #[test]
    #[cfg(feature = "derive")]
    fn derive() {
        use crate::array::StructArray;
        use arrow_array::{cast::AsArray, Array};
        use std::sync::Arc;
        #[derive(crate::ArrayType, Copy, Clone, Debug, Default)]
        struct Unit;

        #[derive(crate::ArrayType, Copy, Clone, Debug, Default)]
        struct NestedUnit(Unit);

        let input = [Unit; 4];
        let array = input.into_iter().collect::<NullArray<Unit>>();
        let arrow_array: arrow_array::NullArray = array.into();
        assert!(arrow_array.data_type().is_null());
        let narrow_array = NullArray::<Unit>::from(arrow_array);
        assert_eq!(narrow_array.len(), 4);

        let input_nested = [NestedUnit(Unit); 4];
        let array_nested = input_nested
            .into_iter()
            .collect::<StructArray<NestedUnit>>();
        let arrow_array_nested: arrow_array::StructArray = array_nested.into();
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
            .collect::<NullArray<_, NonNullable, ArcBuffer>>();
        assert_eq!(
            arrow_array::NullArray::new(null_array_arc.len()).len(),
            INPUT.len()
        );
    }

    #[test]
    fn into() {
        let null_array = arrow_array::NullArray::new(INPUT.len());
        assert_eq!(
            NullArray::<(), NonNullable, crate::arrow::buffer::ScalarBuffer>::from(null_array)
                .into_iter()
                .collect::<Vec<_>>(),
            INPUT
        );
    }
}
