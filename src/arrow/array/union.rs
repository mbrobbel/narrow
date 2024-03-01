//! Interop with [`arrow-rs`] union arrays.

use std::sync::Arc;

use arrow_array::types::Int8Type;
use arrow_schema::{DataType, Field, Fields};

use crate::{
    array::{
        FixedSizePrimitiveArray, Int8Array, SparseLayout, SparseUnionArray, UnionArray,
        UnionArrayType,
    },
    arrow::ArrowArray,
    buffer::BufferType,
    offset::OffsetElement,
};

/// Arrow schema interop trait for the variants of a union array type.
pub trait UnionArrayTypeFields {
    /// Returns the fields of the variants of this union array.
    fn fields() -> Fields;
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > ArrowArray for UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>: UnionArrayTypeFields,
{
    // NOTE: we don't convert to `arrow_array::UnionArray` here to support
    // writing these arrays to parquet files.
    // TODO(mbrobbel): put this choice behind a feature flag.
    type Array = arrow_array::StructArray;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(
            name,
            DataType::Struct(Fields::from(vec![
                Field::new("types", DataType::Int8, false),
                Field::new(
                    "variants",
                    DataType::Struct(<<T as UnionArrayType<VARIANTS>>::Array<
                        Buffer,
                        OffsetItem,
                        SparseLayout,
                    > as UnionArrayTypeFields>::fields()),
                    false,
                ),
            ])),
            false,
        )
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > From<UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>> for arrow_array::StructArray
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>:
        UnionArrayTypeFields + Into<arrow_array::StructArray>,
    arrow_array::PrimitiveArray<Int8Type>: From<FixedSizePrimitiveArray<i8, false, Buffer>>,
{
    fn from(value: UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>) -> Self {
        // Safety:
        // - todo
        unsafe {
            arrow_array::StructArray::new_unchecked(
                Fields::from(vec![
                    Field::new("types", DataType::Int8, false),
                    Field::new(
                        "variants",
                        DataType::Struct(<<T as UnionArrayType<VARIANTS>>::Array<
                            Buffer,
                            OffsetItem,
                            SparseLayout,
                        > as UnionArrayTypeFields>::fields(
                        )),
                        false,
                    ),
                ]),
                vec![
                    Arc::new(arrow_array::PrimitiveArray::from(value.0.types)),
                    Arc::new(value.0.variants.into()),
                ],
                None,
            )
        }
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > From<Arc<dyn arrow_array::Array>>
    for UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    Self: From<arrow_array::StructArray>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::StructArray::from(value.to_data()))
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > From<arrow_array::StructArray> for UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>:
        From<Arc<dyn arrow_array::Array>>,
    Int8Array<false, Buffer>: From<Arc<dyn arrow_array::Array>>,
{
    fn from(value: arrow_array::StructArray) -> Self {
        let (_fields, mut arrays, nulls_opt) = value.into_parts();
        assert_eq!(arrays.len(), 2);
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => UnionArray(SparseUnionArray {
                variants: arrays.pop().expect("an array").into(),
                types: arrays.pop().expect("an array").into(),
            }),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "derive")]
mod tests {
    use super::*;
    use crate::array::DenseLayout;
    use crate::arrow::buffer_builder::ArrowBufferBuilder;
    use arrow_array::Array;

    #[derive(crate::ArrayType)]
    enum FooBar {
        Foo,
        Bar(u8),
        Baz { a: bool },
    }

    #[test]
    fn from() {
        let union_array = [
            FooBar::Foo,
            FooBar::Bar(123),
            FooBar::Baz { a: true },
            FooBar::Foo,
        ]
        .into_iter()
        .collect::<UnionArray<FooBar, 3, SparseLayout, ArrowBufferBuilder>>();
        let struct_array_arrow = arrow_array::StructArray::from(union_array);
        assert_eq!(struct_array_arrow.len(), 4);
        let _rb = arrow_array::RecordBatch::from(struct_array_arrow);
    }
}
