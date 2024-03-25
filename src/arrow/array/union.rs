//! Interop with [`arrow-rs`] union arrays.

use std::sync::Arc;

use arrow_schema::{DataType, Field, Fields, UnionMode};

use crate::{
    array::{
        DenseLayout, FixedSizePrimitiveArray, SparseLayout, UnionArray, UnionArrayType, UnionType,
    },
    buffer::BufferType,
    offset::OffsetElement,
};

/// Mapping between [`UnionType`] and [`UnionMode`].
pub trait UnionLayoutExt: UnionType {
    /// The corresponding [`UnionMode`] for [`UnionType`].
    const MODE: UnionMode;
}

impl UnionLayoutExt for DenseLayout {
    const MODE: UnionMode = UnionMode::Dense;
}

impl UnionLayoutExt for SparseLayout {
    const MODE: UnionMode = UnionMode::Sparse;
}

/// Arrow schema interop trait for the variants of a union array type.
pub trait UnionArrayTypeFields<const VARIANTS: usize> {
    /// Returns the fields of the variants of this union array.
    fn fields() -> Fields;
    /// Returns the type ids of the variants of this union array.
    fn type_ids() -> [i8; VARIANTS];
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        UnionLayout: UnionLayoutExt,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > crate::arrow::Array for UnionArray<T, VARIANTS, UnionLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, UnionLayout>:
        UnionArrayTypeFields<VARIANTS>,
{
    type Array = arrow_array::UnionArray;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(
            name,
            DataType::Union(
                <<T as UnionArrayType<VARIANTS>>::Array<
                    Buffer,
                    OffsetItem,
                    UnionLayout,
                > as UnionArrayTypeFields<VARIANTS>>::type_ids().iter().copied().zip(<<T as UnionArrayType<VARIANTS>>::Array<
                    Buffer,
                    OffsetItem,
                    UnionLayout,
                > as UnionArrayTypeFields<VARIANTS>>::fields().iter().map(Arc::clone)).collect(),
                <UnionLayout as UnionLayoutExt>::MODE
            ),
            false
        )
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > From<UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>> for arrow_array::UnionArray
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>:
        UnionArrayTypeFields<VARIANTS>, // + Into<Vec<(Field, Arc<dyn arrow_array::Array>)>>,
    arrow_buffer::ScalarBuffer<i8>: From<FixedSizePrimitiveArray<i8, false, Buffer>>,
{
    fn from(_value: UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>) -> Self {
        todo!()
        // Safety:
        // - todo
        // unsafe {
        //     arrow_array::UnionArray::new_unchecked(
        //         &<<T as UnionArrayType<VARIANTS>>::Array<
        //             Buffer,
        //             OffsetItem,
        //             SparseLayout,
        //         > as UnionArrayTypeFields<VARIANTS>>::type_ids(),
        //         arrow_buffer::ScalarBuffer::from(value.0.types).into_inner(),
        //         None,
        //     )
        // }
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        UnionLayout: UnionType,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > From<Arc<dyn arrow_array::Array>> for UnionArray<T, VARIANTS, UnionLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    Self: From<arrow_array::StructArray>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::StructArray::from(value.to_data()))
    }
}

#[cfg(test)]
#[cfg(feature = "derive")]
mod tests {
    use crate::Length;

    use super::*;

    #[derive(crate::ArrayType)]
    enum FooBar {
        Foo,
        Bar(u8),
        Baz { a: bool },
    }

    #[test]
    fn from() {
        let sparse_union_array = [
            FooBar::Foo,
            FooBar::Bar(123),
            FooBar::Baz { a: true },
            FooBar::Foo,
        ]
        .into_iter()
        .collect::<UnionArray<FooBar, 3, SparseLayout>>();
        assert_eq!(sparse_union_array.len(), 4);
        // let union_array_arrow = arrow_array::UnionArray::from(sparse_union_array);
        // assert_eq!(arrow_array::Array::len(&union_array_arrow), 4);

        let dense_union_array = [
            FooBar::Foo,
            FooBar::Bar(123),
            FooBar::Baz { a: true },
            FooBar::Foo,
        ]
        .into_iter()
        .collect::<UnionArray<FooBar, 3, DenseLayout>>();
        assert_eq!(dense_union_array.len(), 4);
        // let union_array_arrow = arrow_array::UnionArray::from(dense_union_array);
    }
}
