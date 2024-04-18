//! Interop with [`arrow-rs`] arrays for logical arrays.

use std::sync::Arc;

use crate::{
    array::UnionType,
    arrow::ArrowArray,
    buffer::BufferType,
    logical::{LogicalArray, LogicalArrayType},
    offset::OffsetElement,
    validity::{Nullability, Validity},
};

impl<
        T: LogicalArrayType,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > ArrowArray for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout>:
        Validity<NULLABLE> + ArrowArray,
    T: Nullability<NULLABLE>,
{
    type Array =
        <<T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as ArrowArray>::Array;

    fn as_field(name: &str) -> arrow_schema::Field {
        <<T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as ArrowArray>::as_field(
            name,
        )
    }
}

impl<
        T: LogicalArrayType,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > From<Arc<dyn arrow_array::Array>>
    for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>,
    <<T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Validity<NULLABLE>>::Storage<Buffer>: From<Arc<dyn arrow_array::Array>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self(value.into())
    }
}

impl<
        T: LogicalArrayType,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > From<LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>>
    for arrow_array::FixedSizeListArray
where
    <T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>,
    arrow_array::FixedSizeListArray:
        From<
            <<T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Validity<
                NULLABLE,
            >>::Storage<Buffer>,
        >,
{
    fn from(value: LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>) -> Self {
        value.0.into()
    }
}

impl<
        T: LogicalArrayType,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > From<LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>>
    for arrow_array::FixedSizeBinaryArray
where
    <T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>,
    arrow_array::FixedSizeBinaryArray:
        From<
            <<T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Validity<
                NULLABLE,
            >>::Storage<Buffer>,
        >,
{
    fn from(value: LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>) -> Self {
        value.0.into()
    }
}
