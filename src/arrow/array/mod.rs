//! Interop with [`arrow-array`].

mod boolean;
mod fixed_size_list;
mod fixed_size_primitive;
mod string;
mod r#struct;
use std::sync::Arc;

use arrow_array::types::ArrowPrimitiveType;
pub use r#struct::StructArrayTypeFields;
mod variable_size_list;

use crate::{
    array::{LogicalArray, LogicalArrayType, UnionType},
    buffer::BufferType,
    offset::OffsetElement,
};

use super::ArrowArray;
impl<
        T: LogicalArrayType,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > ArrowArray for LogicalArray<T, false, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout>: ArrowArray,
{
    type Array = <<T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout> as ArrowArray>::Array;

    fn as_field(name: &str) -> arrow_schema::Field {
        // todo add metadata
        <<T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout> as ArrowArray>::as_field(name)
    }
}

impl<
        T: LogicalArrayType,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > From<LogicalArray<T, false, Buffer, OffsetItem, UnionLayout>>
    for arrow_array::FixedSizeListArray
where
    Self: From<<T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout>>,
{
    fn from(value: LogicalArray<T, false, Buffer, OffsetItem, UnionLayout>) -> Self {
        Self::from(value.0)
    }
}

impl<
        U: ArrowPrimitiveType,
        T: LogicalArrayType,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > From<LogicalArray<T, false, Buffer, OffsetItem, UnionLayout>>
    for arrow_array::PrimitiveArray<U>
where
    Self: From<<T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout>>,
{
    fn from(value: LogicalArray<T, false, Buffer, OffsetItem, UnionLayout>) -> Self {
        Self::from(value.0)
    }
}

impl<
        T: LogicalArrayType,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > From<Arc<dyn arrow_array::Array>> for LogicalArray<T, false, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout>:
        From<Arc<dyn arrow_array::Array>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self(value.into())
    }
}
