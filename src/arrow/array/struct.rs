//! Interop with [`arrow-rs`] struct arrays.

use std::sync::Arc;

use arrow_buffer::NullBuffer;
use arrow_schema::{DataType, Field, Fields};

use crate::{
    array::{StructArray, StructArrayType},
    bitmap::Bitmap,
    buffer::{BufferType, VecBuffer},
    nullable::Nullable,
    validity::{Nullability, Validity},
    Length,
};

/// Arrow schema interop trait for the fields of a struct array type.
pub trait StructArrayTypeFields {
    /// Returns the fields of this struct array.
    fn fields() -> Fields;
}

impl<T: StructArrayType, const NULLABLE: bool, Buffer: BufferType> crate::arrow::Array
    for StructArray<T, NULLABLE, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE> + StructArrayTypeFields,
    T: Nullability<NULLABLE>,
{
    type Array = arrow_array::StructArray;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(
            name,
            DataType::Struct(
                <<T as StructArrayType>::Array<Buffer> as StructArrayTypeFields>::fields(),
            ),
            NULLABLE,
        )
    }
}

impl<T: StructArrayType, const NULLABLE: bool, Buffer: BufferType> From<Arc<dyn arrow_array::Array>>
    for StructArray<T, NULLABLE, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>,
    Self: From<arrow_array::StructArray>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::StructArray::from(value.to_data()))
    }
}

impl<T: StructArrayType, const NULLABLE: bool, Buffer: BufferType>
    From<StructArray<T, NULLABLE, Buffer>> for Arc<dyn arrow_array::Array>
where
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>,
    arrow_array::StructArray: From<StructArray<T, NULLABLE, Buffer>>,
{
    fn from(value: StructArray<T, NULLABLE, Buffer>) -> Self {
        Arc::new(arrow_array::StructArray::from(value))
    }
}

impl<T: StructArrayType, Buffer: BufferType> From<StructArray<T, false, Buffer>>
    for arrow_array::StructArray
where
    <T as StructArrayType>::Array<Buffer>:
        StructArrayTypeFields + Into<Vec<Arc<dyn arrow_array::Array>>>,
{
    fn from(value: StructArray<T, false, Buffer>) -> Self {
        // Safety:
        // - struct arrays are valid by construction
        unsafe {
            arrow_array::StructArray::new_unchecked(
                <<T as StructArrayType>::Array<Buffer> as StructArrayTypeFields>::fields(),
                value.0.into(),
                None,
            )
        }
    }
}

impl<T: StructArrayType, Buffer: BufferType> From<StructArray<T, true, Buffer>>
    for arrow_array::StructArray
where
    <T as StructArrayType>::Array<Buffer>:
        StructArrayTypeFields + Into<Vec<Arc<dyn arrow_array::Array>>>,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn from(value: StructArray<T, true, Buffer>) -> Self {
        // Safety:
        // - struct arrays are valid by construction
        unsafe {
            arrow_array::StructArray::new_unchecked(
                <<T as StructArrayType>::Array<Buffer> as StructArrayTypeFields>::fields(),
                value.0.data.into(),
                Some(value.0.validity.into()),
            )
        }
    }
}

/// Panics when there are nulls
impl<T: StructArrayType, Buffer: BufferType> From<arrow_array::StructArray>
    for StructArray<T, false, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: From<Vec<Arc<dyn arrow_array::Array>>>,
{
    fn from(value: arrow_array::StructArray) -> Self {
        let (_fields, arrays, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => StructArray(arrays.into()),
        }
    }
}

impl<T: StructArrayType, Buffer: BufferType> From<arrow_array::StructArray>
    for StructArray<T, true, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: From<Vec<Arc<dyn arrow_array::Array>>> + Length,
    Bitmap<Buffer>: From<NullBuffer> + FromIterator<bool>,
{
    fn from(value: arrow_array::StructArray) -> Self {
        let (_fields, arrays, nulls_opt) = value.into_parts();
        let data = arrays.into();
        match nulls_opt {
            Some(null_buffer) => StructArray(Nullable {
                data,
                validity: null_buffer.into(),
            }),
            None => StructArray::<T, false, Buffer>(data).into(),
        }
    }
}

impl<T: StructArrayType, const NULLABLE: bool, Buffer: BufferType>
    From<StructArray<T, NULLABLE, Buffer>> for arrow_array::RecordBatch
where
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>,
    arrow_array::StructArray: From<StructArray<T, NULLABLE, Buffer>>,
{
    fn from(value: StructArray<T, NULLABLE, Buffer>) -> Self {
        Self::from(arrow_array::StructArray::from(value))
    }
}

impl<T: StructArrayType, const NULLABLE: bool, Buffer: BufferType> From<arrow_array::RecordBatch>
    for StructArray<T, NULLABLE, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>,
    Self: From<arrow_array::StructArray>,
{
    fn from(value: arrow_array::RecordBatch) -> Self {
        Self::from(arrow_array::StructArray::from(value))
    }
}

impl<T: StructArrayType, const NULLABLE: bool, Buffer: BufferType> StructArray<T, NULLABLE, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>,
{
    /// Return the Arrow schema using the fields of this StructArray.
    pub fn schema() -> Arc<arrow_schema::Schema>
    where
        T: StructArrayType,
        <T as StructArrayType>::Array<VecBuffer>: FromIterator<T>,
        <T as StructArrayType>::Array<VecBuffer>: StructArrayTypeFields,
        Vec<Arc<(dyn arrow_array::Array + 'static)>>:
            From<<T as StructArrayType>::Array<VecBuffer>>,
    {
        let dummy_array = ([] as [T; 0]).into_iter().collect::<StructArray<T>>();
        let dummy_batch = arrow_array::RecordBatch::from(dummy_array);
        dummy_batch.schema()
    }
}

#[cfg(test)]
mod tests {

    use arrow_array::{cast::AsArray as _, types::UInt32Type, Array as _};

    use crate::{
        array::{
            union::{self, UnionType},
            ArrayType,
        },
        arrow::buffer::{BufferBuilder, ScalarBuffer},
        bitmap::ValidityBitmap,
        buffer::Buffer as _,
        offset::{self, OffsetElement},
    };

    use super::*;

    #[derive(Copy, Clone, Default, Debug, PartialEq)]
    struct Foo {
        a: u32,
    }
    struct FooArray<Buffer: BufferType> {
        a: <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>,
    }
    impl ArrayType<Foo> for Foo {
        type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
            StructArray<Foo, false, Buffer>;
    }
    impl ArrayType<Foo> for Option<Foo> {
        type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
            StructArray<Foo, true, Buffer>;
    }
    impl<Buffer: BufferType> Default for FooArray<Buffer>
    where
        <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>: Default,
    {
        fn default() -> Self {
            Self {
                a: <u32 as ArrayType<u32>>::Array::<Buffer, offset::NA, union::NA>::default(),
            }
        }
    }
    impl<Buffer: BufferType> Extend<Foo> for FooArray<Buffer>
    where
        <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>: Extend<u32>,
    {
        fn extend<I: IntoIterator<Item = Foo>>(&mut self, iter: I) {
            iter.into_iter().for_each(|Foo { a }| {
                self.a.extend(std::iter::once(a));
            });
        }
    }
    impl<Buffer: BufferType> FromIterator<Foo> for FooArray<Buffer>
    where
        <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>: Default + Extend<u32>,
    {
        fn from_iter<T: IntoIterator<Item = Foo>>(iter: T) -> Self {
            let (a, _): (_, Vec<_>) = iter.into_iter().map(|Foo { a }| (a, ())).unzip();
            Self { a }
        }
    }
    struct FooArrayIter<Buffer: BufferType> where <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>: IntoIterator {
        a: <<u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA> as IntoIterator>::IntoIter,
    }
    impl<Buffer: BufferType> Iterator for FooArrayIter<Buffer>
    where
        <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>: IntoIterator<Item = u32>,
    {
        type Item = Foo;

        fn next(&mut self) -> Option<Self::Item> {
            self.a.next().map(|a| Foo { a })
        }
    }
    impl<Buffer: BufferType> IntoIterator for FooArray<Buffer>
    where
        <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>: IntoIterator<Item = u32>,
    {
        type Item = Foo;
        type IntoIter = FooArrayIter<Buffer>;

        fn into_iter(self) -> Self::IntoIter {
            Self::IntoIter {
                a: self.a.into_iter(),
            }
        }
    }
    impl<Buffer: BufferType> Length for FooArray<Buffer> {
        fn len(&self) -> usize {
            self.a.len()
        }
    }
    impl StructArrayType for Foo {
        type Array<Buffer: BufferType> = FooArray<Buffer>;
    }
    impl<Buffer: BufferType> StructArrayTypeFields for FooArray<Buffer> {
        fn fields() -> Fields {
            Fields::from(vec![Field::new("a", DataType::UInt32, false)])
        }
    }
    impl<Buffer: BufferType> From<FooArray<Buffer>> for Vec<Arc<dyn arrow_array::Array>>
    where
        <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>:
            Into<<<u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA> as crate::arrow::Array>::Array>,
    {
        fn from(value: FooArray<Buffer>) -> Self {
            vec![Arc::<
                <<u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA> as crate::arrow::Array>::Array,
            >::new(value.a.into())]
        }
    }
    impl<Buffer: BufferType> From<Vec<Arc<dyn arrow_array::Array>>> for FooArray<Buffer>
    where
        <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>:
            From<Arc<dyn arrow_array::Array>>,
    {
        fn from(value: Vec<Arc<dyn arrow_array::Array>>) -> Self {
            let mut arrays = value.into_iter();
            let result = Self {
                a: arrays.next().expect("an array").into(),
            };
            assert!(arrays.next().is_none());
            result
        }
    }

    #[test]
    fn from() {
        let struct_array = [Foo { a: 1 }, Foo { a: 2 }]
            .into_iter()
            .collect::<StructArray<Foo, false, BufferBuilder>>();
        let struct_array_arrow = arrow_array::StructArray::from(struct_array);
        assert_eq!(struct_array_arrow.len(), 2);

        let struct_array_nullable = [Some(Foo { a: 1234 }), None]
            .into_iter()
            .collect::<StructArray<Foo, true, BufferBuilder>>();
        let struct_array_arrow_nullable = arrow_array::StructArray::from(struct_array_nullable);
        assert_eq!(struct_array_arrow_nullable.len(), 2);
        assert!(struct_array_arrow_nullable.is_valid(0));
        assert!(struct_array_arrow_nullable.is_null(1));
        assert_eq!(
            struct_array_arrow_nullable
                .column(0)
                .as_primitive::<UInt32Type>()
                .values()
                .as_slice(),
            [1234, u32::default()]
        );

        // And convert back
        let roundtrip: StructArray<Foo, true, ScalarBuffer> = struct_array_arrow_nullable.into();
        assert_eq!(roundtrip.0.data.a.0, [1234, u32::default()]);
    }

    #[test]
    fn into_nullable() {
        let struct_array = [Foo { a: 1 }, Foo { a: 2 }]
            .into_iter()
            .collect::<StructArray<Foo, false, BufferBuilder>>();
        let struct_array_arrow = arrow_array::StructArray::from(struct_array);
        assert!(!StructArray::<Foo, true, ScalarBuffer>::from(struct_array_arrow).any_null());
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let struct_array_nullable = [Some(Foo { a: 1234 }), None]
            .into_iter()
            .collect::<StructArray<Foo, true, BufferBuilder>>();
        let struct_array_arrow_nullable = arrow_array::StructArray::from(struct_array_nullable);
        let _ = StructArray::<Foo, false, ScalarBuffer>::from(struct_array_arrow_nullable);
    }

    #[test]
    fn into() {
        let struct_array = [Foo { a: 1 }, Foo { a: 2 }]
            .into_iter()
            .collect::<StructArray<Foo, false, BufferBuilder>>();
        let struct_array_arrow = arrow_array::StructArray::from(struct_array);
        assert_eq!(
            StructArray::<Foo, false, ScalarBuffer>::from(struct_array_arrow)
                .0
                .a
                .0,
            [1, 2]
        );
        let struct_array_nullable = [Some(Foo { a: 1234 }), None]
            .into_iter()
            .collect::<StructArray<Foo, true, BufferBuilder>>();
        let struct_array_arrow_nullable = arrow_array::StructArray::from(struct_array_nullable);
        assert_eq!(
            StructArray::<Foo, true, ScalarBuffer>::from(struct_array_arrow_nullable)
                .0
                .data
                .a
                .0,
            [1234, u32::default()]
        );
    }

    #[test]
    fn into_iter() {
        let input = [Foo { a: 1 }, Foo { a: 2345 }];
        let struct_array = input.into_iter().collect::<StructArray<Foo, false>>();
        let vec = struct_array.into_iter().collect::<Vec<Foo>>();
        assert_eq!(input.as_slice(), vec.as_slice());

        let input_nullable = [Some(Foo { a: 1 }), None, Some(Foo { a: 2345 })];
        let struct_array_nullable = input_nullable
            .into_iter()
            .collect::<StructArray<Foo, true>>();
        let vec_nullable = struct_array_nullable
            .into_iter()
            .collect::<Vec<Option<Foo>>>();
        assert_eq!(input_nullable.as_slice(), vec_nullable.as_slice());
    }

    #[test]
    #[cfg(feature = "derive")]
    fn derived() {
        use crate::Length;

        use super::*;
        use arrow_array::Array as _;

        #[derive(crate::ArrayType)]
        struct Foo<T>(T, u32);

        let struct_array = [Foo(1_i32, 2), Foo(3, 4)]
            .into_iter()
            .collect::<StructArray<Foo<_>, false>>();
        let struct_array_arrow = arrow_array::StructArray::from(struct_array);
        assert_eq!(struct_array_arrow.len(), 2);

        let struct_array_roundtrip: StructArray<Foo<i32>> = struct_array_arrow.into();
        assert_eq!(struct_array_roundtrip.len(), 2);
    }

    #[cfg(feature = "derive")]
    #[derive(narrow_derive::ArrayType)]
    struct Bar {
        a: u8,
        b: Option<Vec<i32>>,
    }

    #[test]
    #[cfg(feature = "derive")]
    fn schema() {
        let schema = StructArray::<Bar>::schema();

        let fields = schema.fields();
        assert_eq!(fields.len(), 2);

        assert_eq!(fields[0].name(), "a");
        assert_eq!(fields[0].is_nullable(), false);
        assert_eq!(*fields[0].data_type(), arrow_schema::DataType::UInt8);

        assert_eq!(fields[1].name(), "b");
        assert_eq!(fields[1].is_nullable(), true);
        assert_eq!(
            *fields[1].data_type(),
            arrow_schema::DataType::List(Arc::new(Field::new(
                "item",
                arrow_schema::DataType::Int32,
                false
            )))
        );
    }
}
