//! Interop with [`arrow-rs`] union arrays.

use std::sync::Arc;

use arrow_schema::{DataType, Field, Fields, UnionMode};

use crate::{
    array::{
        DenseLayout, DenseUnionArray, FixedSizePrimitiveArray, SparseLayout, SparseUnionArray,
        UnionArray, UnionArrayType, UnionType,
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
        UnionArrayTypeFields<VARIANTS> + Into<Vec<(Field, Arc<dyn arrow_array::Array>)>>,
    arrow_buffer::ScalarBuffer<i8>: From<FixedSizePrimitiveArray<i8, false, Buffer>>,
{
    fn from(value: UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>) -> Self {
        // Safety:
        // - todo
        unsafe {
            arrow_array::UnionArray::new_unchecked(
                &<<T as UnionArrayType<VARIANTS>>::Array<
                    Buffer,
                    OffsetItem,
                    SparseLayout,
                > as UnionArrayTypeFields<VARIANTS>>::type_ids(),
                arrow_buffer::ScalarBuffer::from(value.0.types).into_inner(),
                None,
                value.0.variants.into()
            )
        }
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > From<UnionArray<T, VARIANTS, DenseLayout, Buffer, OffsetItem>> for arrow_array::UnionArray
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, DenseLayout>:
        UnionArrayTypeFields<VARIANTS> + Into<Vec<(Field, Arc<dyn arrow_array::Array>)>>,
    arrow_buffer::ScalarBuffer<i8>: From<FixedSizePrimitiveArray<i8, false, Buffer>>,
    arrow_buffer::ScalarBuffer<i32>: From<FixedSizePrimitiveArray<i32, false, Buffer>>,
{
    fn from(value: UnionArray<T, VARIANTS, DenseLayout, Buffer, OffsetItem>) -> Self {
        // Safety:
        // - todo
        unsafe {
            arrow_array::UnionArray::new_unchecked(
                &<<T as UnionArrayType<VARIANTS>>::Array<
                    Buffer,
                    OffsetItem,
                    DenseLayout,
                > as UnionArrayTypeFields<VARIANTS>>::type_ids(),
                arrow_buffer::ScalarBuffer::from(value.0.types).into_inner(),
                Some(arrow_buffer::ScalarBuffer::<i32>::from(value.0.offsets).into_inner()),
                value.0.variants.into()
            )
        }
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > From<arrow_array::UnionArray> for UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    FixedSizePrimitiveArray<i8, false, Buffer>: From<arrow_buffer::ScalarBuffer<i8>>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>:
        FromIterator<Arc<dyn arrow_array::Array>>,
{
    fn from(value: arrow_array::UnionArray) -> Self {
        let (types, offsets_opt, _field_type_ids, variants) = value.into_parts();
        match offsets_opt {
            Some(_) => panic!("expected array without offsets"),
            None => Self(SparseUnionArray {
                variants: variants.into_iter().map(|(_, array)| array).collect(),
                types: types.into(),
            }),
        }
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > From<arrow_array::UnionArray> for UnionArray<T, VARIANTS, DenseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    FixedSizePrimitiveArray<i8, false, Buffer>: From<arrow_buffer::ScalarBuffer<i8>>,
    FixedSizePrimitiveArray<i32, false, Buffer>: From<arrow_buffer::ScalarBuffer<i32>>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, DenseLayout>:
        FromIterator<Arc<dyn arrow_array::Array>>,
{
    fn from(value: arrow_array::UnionArray) -> Self {
        let (types, offsets_opt, _field_types_ids, variants) = value.into_parts();
        match offsets_opt {
            None => panic!("expected array with offsets"),
            Some(offsets) => Self(DenseUnionArray {
                variants: variants.into_iter().map(|(_, array)| array).collect(),
                offsets: offsets.into(),
                types: types.into(),
            }),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "derive")]
mod tests {
    use crate::Length;

    use super::*;

    #[derive(crate::ArrayType, Clone)]
    enum FooBar {
        Foo,
        Bar(u8),
        Baz { a: bool },
    }

    #[test]
    fn from() {
        let input = [
            FooBar::Foo,
            FooBar::Bar(123),
            FooBar::Baz { a: true },
            FooBar::Foo,
        ];
        let sparse_union_array = input
            .clone()
            .into_iter()
            .collect::<UnionArray<FooBar, 3, SparseLayout>>();
        assert_eq!(sparse_union_array.len(), 4);
        let union_array_arrow = arrow_array::UnionArray::from(sparse_union_array);
        assert_eq!(arrow_array::Array::len(&union_array_arrow), 4);

        let dense_union_array = input
            .into_iter()
            .collect::<UnionArray<FooBar, 3, DenseLayout>>();
        assert_eq!(dense_union_array.len(), 4);
        let dense_union_array_arrow = arrow_array::UnionArray::from(dense_union_array);
        assert_eq!(arrow_array::Array::len(&dense_union_array_arrow), 4);
    }

    #[test]
    fn into() {
        let input = [
            FooBar::Foo,
            FooBar::Bar(123),
            FooBar::Baz { a: true },
            FooBar::Foo,
        ];
        let sparse_union_array = input
            .clone()
            .into_iter()
            .collect::<UnionArray<FooBar, 3, SparseLayout>>();
        let union_array_arrow = arrow_array::UnionArray::from(sparse_union_array);
        let narrow_union_array: UnionArray<FooBar, 3, SparseLayout> = union_array_arrow.into();
        assert_eq!(narrow_union_array.len(), 4);

        let dense_union_array = input
            .into_iter()
            .collect::<UnionArray<FooBar, 3, DenseLayout>>();
        let dense_union_array_arrow = arrow_array::UnionArray::from(dense_union_array);
        let narrow_dense_union_array: UnionArray<FooBar, 3, DenseLayout> =
            dense_union_array_arrow.into();
        assert_eq!(narrow_dense_union_array.len(), 4);
    }

    #[test]
    #[should_panic(expected = "expected array with offsets")]
    fn into_dense() {
        let input = [
            FooBar::Foo,
            FooBar::Bar(123),
            FooBar::Baz { a: true },
            FooBar::Foo,
        ];
        let union_array = input
            .clone()
            .into_iter()
            .collect::<UnionArray<FooBar, 3, SparseLayout>>();
        let union_array_arrow = arrow_array::UnionArray::from(union_array);
        let _: UnionArray<FooBar, 3, DenseLayout> = union_array_arrow.into();
    }

    #[test]
    #[should_panic(expected = "expected array without offsets")]
    fn into_sparse() {
        let input = [
            FooBar::Foo,
            FooBar::Bar(123),
            FooBar::Baz { a: true },
            FooBar::Foo,
        ];
        let union_array = input
            .clone()
            .into_iter()
            .collect::<UnionArray<FooBar, 3, DenseLayout>>();
        let union_array_arrow = arrow_array::UnionArray::from(union_array);
        let _array: UnionArray<FooBar, 3, SparseLayout> = union_array_arrow.into();
    }

    #[test]
    #[should_panic(expected = "NullArray data type should be Null")]
    fn wrong_conversion() {
        #[derive(crate::ArrayType)]
        enum Bar {
            A,
            B,
        }
        let input = [
            FooBar::Foo,
            FooBar::Bar(123),
            FooBar::Baz { a: true },
            FooBar::Foo,
        ];
        let union_array = input
            .clone()
            .into_iter()
            .collect::<UnionArray<FooBar, 3, DenseLayout>>();
        let union_array_arrow = arrow_array::UnionArray::from(union_array);
        let _array: UnionArray<Bar, 2, DenseLayout> = union_array_arrow.into();
    }

    #[test]
    #[should_panic(expected = "not enough variant data arrays, expected 6")]
    fn wrong_variants() {
        #[derive(crate::ArrayType)]
        enum One {
            A,
        }
        #[derive(crate::ArrayType)]
        enum Bar {
            A,
            B,
            C,
            D,
            E,
            F,
        }
        let input = [One::A];
        let union_array = input
            .into_iter()
            .collect::<UnionArray<One, 1, SparseLayout>>();
        let union_array_arrow = arrow_array::UnionArray::from(union_array);
        let _array: UnionArray<Bar, 6, SparseLayout> = union_array_arrow.into();
    }
}
