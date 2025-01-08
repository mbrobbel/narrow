//! Interop with [`arrow-rs`] union arrays.

use std::sync::Arc;

use arrow_schema::{DataType, Field, Fields, UnionMode};

use crate::{
    array::{
        DenseLayout, DenseUnionArray, FixedSizePrimitiveArray, SparseLayout, SparseUnionArray,
        UnionArray, UnionArrayType, UnionType,
    },
    buffer::BufferType,
    offset::Offset,
    NonNullable,
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
        OffsetItem: Offset,
    > crate::arrow::Array for UnionArray<T, VARIANTS, UnionLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, UnionLayout>:
        UnionArrayTypeFields<VARIANTS>,
{
    type Array = arrow_array::UnionArray;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(name, Self::data_type(), false)
    }

    fn data_type() -> arrow_schema::DataType {
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
        )
    }
}


impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: Offset,
    > Into<arrow_array::UnionArray> for UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>:
        UnionArrayTypeFields<VARIANTS> + Into<Vec<Arc<dyn arrow_array::Array>>>,
    FixedSizePrimitiveArray<i8, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<i8>>,
    UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>: crate::arrow::Array,
{
    fn into(self) -> arrow_array::UnionArray {
        let union_fields = match < UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem> as crate::arrow::Array>::as_field("").data_type() {
            &DataType::Union(ref fields, _mode) => fields.to_owned(),
            _ => unreachable!(),
        };
        // Safety:
        // - todo
        unsafe {
            arrow_array::UnionArray::new_unchecked(
                union_fields,
                Into::<arrow_buffer::ScalarBuffer<i8>>::into(self.0.types),
                None,
                self.0.variants.into(),
            )
        }
    }
}


impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: Offset,
    > Into<Arc<dyn arrow_array::Array>>
    for UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>:
        UnionArrayTypeFields<VARIANTS> + Into<Vec<Arc<dyn arrow_array::Array>>>,
    FixedSizePrimitiveArray<i8, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<i8>>,
    UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>: crate::arrow::Array,
{
    fn into(self) -> Arc<dyn arrow_array::Array> {
        Arc::new(Into::<arrow_array::UnionArray>::into(self))
    }
}


impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: Offset,
    > Into<arrow_array::UnionArray> for UnionArray<T, VARIANTS, DenseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, DenseLayout>:
        UnionArrayTypeFields<VARIANTS> + Into<Vec<Arc<dyn arrow_array::Array>>>,
    FixedSizePrimitiveArray<i8, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<i8>>,
    FixedSizePrimitiveArray<i32, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<i32>>,
    UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>: crate::arrow::Array,
{
    fn into(self) -> arrow_array::UnionArray {
        let union_fields = match < UnionArray<T, VARIANTS, DenseLayout, Buffer, OffsetItem> as crate::arrow::Array>::as_field("").data_type() {
            &DataType::Union(ref fields, _mode) => fields.to_owned(),
            _ => unreachable!(),
        };
        // Safety:
        // - todo
        unsafe {
            arrow_array::UnionArray::new_unchecked(
                union_fields,
                Into::<arrow_buffer::ScalarBuffer<i8>>::into(self.0.types),
                Some(Into::<arrow_buffer::ScalarBuffer<i32>>::into(
                    self.0.offsets,
                )),
                self.0.variants.into(),
            )
        }
    }
}


impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: Offset,
    > Into<Arc<dyn arrow_array::Array>> for UnionArray<T, VARIANTS, DenseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, DenseLayout>:
        UnionArrayTypeFields<VARIANTS> + Into<Vec<Arc<dyn arrow_array::Array>>>,
    FixedSizePrimitiveArray<i8, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<i8>>,
    FixedSizePrimitiveArray<i32, NonNullable, Buffer>: Into<arrow_buffer::ScalarBuffer<i32>>,
    UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>: crate::arrow::Array,
{
    fn into(self) -> Arc<dyn arrow_array::Array> {
        Arc::new(Into::<arrow_array::UnionArray>::into(self))
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: Offset,
    > From<arrow_array::UnionArray> for UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    FixedSizePrimitiveArray<i8, NonNullable, Buffer>: From<arrow_buffer::ScalarBuffer<i8>>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>:
        FromIterator<Arc<dyn arrow_array::Array>>,
{
    fn from(value: arrow_array::UnionArray) -> Self {
        let (_union_fields, type_ids, offsets_opt, variants) = value.into_parts();
        match offsets_opt {
            Some(_) => panic!("expected array without offsets"),
            None => Self(SparseUnionArray {
                variants: variants.into_iter().collect(),
                types: type_ids.into(),
            }),
        }
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: Offset,
    > From<Arc<dyn arrow_array::Array>>
    for UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    FixedSizePrimitiveArray<i8, NonNullable, Buffer>: From<arrow_buffer::ScalarBuffer<i8>>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>:
        FromIterator<Arc<dyn arrow_array::Array>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        let array = arrow_array::UnionArray::from(value.to_data());
        Self::from(array)
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: Offset,
    > From<arrow_array::UnionArray> for UnionArray<T, VARIANTS, DenseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    FixedSizePrimitiveArray<i8, NonNullable, Buffer>: From<arrow_buffer::ScalarBuffer<i8>>,
    FixedSizePrimitiveArray<i32, NonNullable, Buffer>: From<arrow_buffer::ScalarBuffer<i32>>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, DenseLayout>:
        FromIterator<Arc<dyn arrow_array::Array>>,
{
    fn from(value: arrow_array::UnionArray) -> Self {
        let (_union_fields, type_ids, offsets_opt, variants) = value.into_parts();
        match offsets_opt {
            None => panic!("expected array with offsets"),
            Some(offsets) => Self(DenseUnionArray {
                variants: variants.into_iter().collect(),
                offsets: offsets.into(),
                types: type_ids.into(),
            }),
        }
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: Offset,
    > From<Arc<dyn arrow_array::Array>> for UnionArray<T, VARIANTS, DenseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    FixedSizePrimitiveArray<i8, NonNullable, Buffer>: From<arrow_buffer::ScalarBuffer<i8>>,
    FixedSizePrimitiveArray<i32, NonNullable, Buffer>: From<arrow_buffer::ScalarBuffer<i32>>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, DenseLayout>:
        FromIterator<Arc<dyn arrow_array::Array>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        let array = arrow_array::UnionArray::from(value.to_data());
        Self::from(array)
    }
}

#[cfg(test)]
#[cfg(feature = "derive")]
mod tests {
    use arrow_array::RecordBatch;

    use crate::{array::StructArray, Length};

    use super::*;

    #[derive(crate::ArrayType, Clone, Debug, PartialEq)]
    enum FooBar {
        Foo,
        Bar(u8),
        Baz { a: bool },
    }

    #[derive(crate::ArrayType, Clone, Debug, PartialEq)]
    struct Wrap(FooBar);

    #[test]
    fn via_dyn_array() {
        let input = [Wrap(FooBar::Foo), Wrap(FooBar::Bar(123))];
        let struct_array = input.clone().into_iter().collect::<StructArray<Wrap>>();
        let record_batch: RecordBatch = struct_array.into();
        let read = StructArray::<Wrap>::from(record_batch);
        assert_eq!(read.into_iter().collect::<Vec<_>>(), input);
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
        let union_array_arrow: arrow_array::UnionArray = sparse_union_array.into();
        assert_eq!(arrow_array::Array::len(&union_array_arrow), 4);

        let dense_union_array = input
            .into_iter()
            .collect::<UnionArray<FooBar, 3, DenseLayout>>();
        assert_eq!(dense_union_array.len(), 4);
        let dense_union_array_arrow: arrow_array::UnionArray = dense_union_array.into();
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
        let union_array_arrow: arrow_array::UnionArray = sparse_union_array.into();
        let narrow_union_array: UnionArray<FooBar, 3, SparseLayout> = union_array_arrow.into();
        assert_eq!(narrow_union_array.len(), 4);

        let dense_union_array = input
            .into_iter()
            .collect::<UnionArray<FooBar, 3, DenseLayout>>();
        let dense_union_array_arrow: arrow_array::UnionArray = dense_union_array.into();
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
        let union_array_arrow: arrow_array::UnionArray = union_array.into();
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
        let union_array_arrow: arrow_array::UnionArray = union_array.into();
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
        let union_array_arrow: arrow_array::UnionArray = union_array.into();
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
        let union_array_arrow: arrow_array::UnionArray = union_array.into();
        let _array: UnionArray<Bar, 6, SparseLayout> = union_array_arrow.into();
    }
}
