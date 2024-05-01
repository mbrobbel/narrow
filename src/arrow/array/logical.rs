//! Interop with [`arrow-rs`] arrays for logical arrays.

use std::sync::Arc;

use crate::{
    array::{ArrayType, UnionType},
    buffer::BufferType,
    logical::{LogicalArray, LogicalArrayType},
    offset::OffsetElement,
    validity::Nullability,
};

// TODO(mbrobbel): add field metadata trait

impl<
        T: LogicalArrayType<T>,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > crate::arrow::Array for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    <T as LogicalArrayType<T>>::ArrayType: Nullability<NULLABLE>,
    <<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item:
        ArrayType<<T as LogicalArrayType<T>>::ArrayType>,
    <<<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout>: crate::arrow::Array,
{
    type Array =
        <<<<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item as ArrayType<
            <T as LogicalArrayType<T>>::ArrayType,
        >>::Array<Buffer, OffsetItem, UnionLayout> as crate::arrow::Array>::Array;

    fn as_field(name: &str) -> arrow_schema::Field {
        <<<<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item as ArrayType<
            <T as LogicalArrayType<T>>::ArrayType,
        >>::Array<Buffer, OffsetItem, UnionLayout> as crate::arrow::Array>::as_field(name)
    }
}

impl<
        T: LogicalArrayType<T>,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > From<Arc<dyn arrow_array::Array>>
    for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    <T as LogicalArrayType<T>>::ArrayType: Nullability<NULLABLE>,
    <<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item:
        ArrayType<<T as LogicalArrayType<T>>::ArrayType>,
    <<<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout>: From<Arc<dyn arrow_array::Array>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self(value.into())
    }
}

impl<
        T: LogicalArrayType<T>,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > From<LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>>
    for Arc<dyn arrow_array::Array>
where
    Option<T>: ArrayType<T>,
    <T as LogicalArrayType<T>>::ArrayType: Nullability<NULLABLE>,
    <<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item:
        ArrayType<<T as LogicalArrayType<T>>::ArrayType>,
    <<<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout>: Into<Arc<dyn arrow_array::Array>>,
{
    fn from(value: LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>) -> Self {
        Arc::new(value.0.into())
    }
}

impl<
        T: LogicalArrayType<T>,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > From<LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>>
    for arrow_array::FixedSizeListArray
where
    Option<T>: ArrayType<T>,
    <T as LogicalArrayType<T>>::ArrayType: Nullability<NULLABLE>,
    <<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item:
        ArrayType<<T as LogicalArrayType<T>>::ArrayType>,
    arrow_array::FixedSizeListArray: From<
        <<<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item as ArrayType<
            <T as LogicalArrayType<T>>::ArrayType,
        >>::Array<Buffer, OffsetItem, UnionLayout>,
    >,
{
    fn from(value: LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>) -> Self {
        value.0.into()
    }
}

impl<
        T: LogicalArrayType<T>,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > From<LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>>
    for arrow_array::FixedSizeBinaryArray
where
    Option<T>: ArrayType<T>,
    <T as LogicalArrayType<T>>::ArrayType: Nullability<NULLABLE>,
    <<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item:
        ArrayType<<T as LogicalArrayType<T>>::ArrayType>,
    arrow_array::FixedSizeBinaryArray: From<
        <<<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item as ArrayType<
            <T as LogicalArrayType<T>>::ArrayType,
        >>::Array<Buffer, OffsetItem, UnionLayout>,
    >,
{
    fn from(value: LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>) -> Self {
        value.0.into()
    }
}
