//! Interop with [`arrow-rs`] arrays for logical arrays.

use std::sync::Arc;

use arrow_array::OffsetSizeTrait;

use crate::{
    array::{ArrayType, NullableArrayTypeOf, UnionType},
    buffer::BufferType,
    logical::{LogicalArray, LogicalArrayType},
    nullability::Nullability,
    offset::Offset,
};

// TODO(mbrobbel): add field metadata trait

impl<
        T: LogicalArrayType<T>,
        Nullable: Nullability,
        Buffer: BufferType,
        OffsetItem: Offset,
        UnionLayout: UnionType,
    > crate::arrow::Array for LogicalArray<T, Nullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Nullable::Item<T::ArrayType>:
        ArrayType<T::ArrayType, Array<Buffer, OffsetItem, UnionLayout>: crate::arrow::Array>,
{
    type Array = <NullableArrayTypeOf<Nullable, T::ArrayType, Buffer, OffsetItem, UnionLayout> as crate::arrow::Array>::Array;

    fn as_field(name: &str) -> arrow_schema::Field {
        <NullableArrayTypeOf<Nullable, T::ArrayType, Buffer, OffsetItem, UnionLayout> as crate::arrow::Array>::as_field(name)
    }

    fn data_type() -> arrow_schema::DataType {
        <NullableArrayTypeOf<Nullable, T::ArrayType, Buffer, OffsetItem, UnionLayout> as crate::arrow::Array>::data_type()
    }
}

impl<
        T: LogicalArrayType<T>,
        Nullable: Nullability,
        Buffer: BufferType,
        OffsetItem: Offset,
        UnionLayout: UnionType,
    > From<Arc<dyn arrow_array::Array>>
    for LogicalArray<T, Nullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Nullable::Item<T::ArrayType>: ArrayType<T::ArrayType>,
    crate::bitmap::Bitmap<Buffer>: FromIterator<bool>,
    <Nullable::Item<T::ArrayType> as ArrayType<T::ArrayType>>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: From<Arc<dyn arrow_array::Array>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self(value.into())
    }
}


impl<
        T: LogicalArrayType<T>,
        Nullable: Nullability,
        Buffer: BufferType,
        OffsetItem: Offset,
        UnionLayout: UnionType,
    > Into<Arc<dyn arrow_array::Array>>
    for LogicalArray<T, Nullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Nullable::Item<T::ArrayType>: ArrayType<T::ArrayType>,
    <Nullable::Item<T::ArrayType> as ArrayType<T::ArrayType>>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: Into<Arc<dyn arrow_array::Array>>,
{
    fn into(self) -> Arc<dyn arrow_array::Array> {
        Arc::new(self.0.into())
    }
}


impl<
        T: LogicalArrayType<T>,
        Nullable: Nullability,
        Buffer: BufferType,
        OffsetItem: Offset,
        UnionLayout: UnionType,
    > Into<arrow_array::FixedSizeListArray>
    for LogicalArray<T, Nullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Nullable::Item<T::ArrayType>: ArrayType<T::ArrayType>,
    <Nullable::Item<T::ArrayType> as ArrayType<T::ArrayType>>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: Into<arrow_array::FixedSizeListArray>,
{
    fn into(self) -> arrow_array::FixedSizeListArray {
        self.0.into()
    }
}


impl<
        T: LogicalArrayType<T>,
        Nullable: Nullability,
        Buffer: BufferType,
        OffsetItem: Offset,
        UnionLayout: UnionType,
    > Into<arrow_array::FixedSizeBinaryArray>
    for LogicalArray<T, Nullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Nullable::Item<T::ArrayType>: ArrayType<T::ArrayType>,
    <Nullable::Item<T::ArrayType> as ArrayType<T::ArrayType>>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: Into<arrow_array::FixedSizeBinaryArray>,
{
    fn into(self) -> arrow_array::FixedSizeBinaryArray {
        self.0.into()
    }
}


impl<
        T: LogicalArrayType<T>,
        Nullable: Nullability,
        Buffer: BufferType,
        OffsetItem: Offset,
        UnionLayout: UnionType,
        O: OffsetSizeTrait,
    > Into<arrow_array::GenericListArray<O>>
    for LogicalArray<T, Nullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Nullable::Item<T::ArrayType>: ArrayType<T::ArrayType>,
    <Nullable::Item<T::ArrayType> as ArrayType<T::ArrayType>>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: Into<arrow_array::GenericListArray<O>>,
{
    fn into(self) -> arrow_array::GenericListArray<O> {
        self.0.into()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "derive")]
    fn optional_variable_size_list_logical() {
        use crate::array::{StructArray, VariableSizeBinary};

        #[derive(crate::ArrayType)]
        struct Foo {
            items: Option<Vec<VariableSizeBinary>>,
        }

        let input = [Foo { items: None }];
        let array = input.into_iter().collect::<StructArray<Foo>>();
        let record_batch: arrow_array::RecordBatch = array.into();
        assert_eq!(record_batch.num_rows(), 1);
    }
}
