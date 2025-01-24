//! Interop with [`arrow-rs`] struct arrays.

use std::sync::Arc;

use arrow_buffer::NullBuffer;
use arrow_schema::{DataType, Field, Fields};

use crate::{
    array::{StructArray, StructArrayType},
    bitmap::Bitmap,
    buffer::BufferType,
    nullability::{NonNullable, Nullability, Nullable},
    validity::Validity,
    Length,
};

/// Arrow schema interop trait for the fields of a struct array type.
pub trait StructArrayTypeFields {
    /// The names of the fields.
    const NAMES: &'static [&'static str];

    /// Returns the fields of this struct array.
    fn fields() -> Fields;
}

impl<T: StructArrayType, Nullable: Nullability, Buffer: BufferType> crate::arrow::Array
    for StructArray<T, Nullable, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: StructArrayTypeFields,
{
    type Array = arrow_array::StructArray;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(name, Self::data_type(), Nullable::NULLABLE)
    }

    fn data_type() -> arrow_schema::DataType {
        DataType::Struct(<<T as StructArrayType>::Array<Buffer> as StructArrayTypeFields>::fields())
    }
}

impl<T: StructArrayType, Nullable: Nullability, Buffer: BufferType>
    From<Arc<dyn arrow_array::Array>> for StructArray<T, Nullable, Buffer>
where
    Self: From<arrow_array::StructArray>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::StructArray::from(value.to_data()))
    }
}

impl<T: StructArrayType, Nullable: Nullability, Buffer: BufferType>
    From<StructArray<T, Nullable, Buffer>> for Arc<dyn arrow_array::Array>
where
    arrow_array::StructArray: From<StructArray<T, Nullable, Buffer>>,
{
    fn from(value: StructArray<T, Nullable, Buffer>) -> Self {
        Arc::new(arrow_array::StructArray::from(value))
    }
}

impl<T: StructArrayType, Buffer: BufferType> From<StructArray<T, NonNullable, Buffer>>
    for arrow_array::StructArray
where
    <T as StructArrayType>::Array<Buffer>:
        StructArrayTypeFields + Into<Vec<Arc<dyn arrow_array::Array>>>,
{
    fn from(value: StructArray<T, NonNullable, Buffer>) -> Self {
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

impl<T: StructArrayType, Buffer: BufferType> From<StructArray<T, Nullable, Buffer>>
    for arrow_array::StructArray
where
    <T as StructArrayType>::Array<Buffer>:
        StructArrayTypeFields + Into<Vec<Arc<dyn arrow_array::Array>>>,
    Bitmap<Buffer>: Into<NullBuffer>,
{
    fn from(value: StructArray<T, Nullable, Buffer>) -> Self {
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
    for StructArray<T, NonNullable, Buffer>
where
    <T as StructArrayType>::Array<Buffer>:
        From<Vec<Arc<dyn arrow_array::Array>>> + StructArrayTypeFields,
{
    fn from(value: arrow_array::StructArray) -> Self {
        let (fields, arrays, nulls_opt) = value.into_parts();
        // Project
        let projected = <<T as StructArrayType>::Array<Buffer> as StructArrayTypeFields>::NAMES
            .iter()
            .map(|field| {
                fields
                    .find(field)
                    .unwrap_or_else(|| panic!("expected struct array with field: {field}"))
            })
            .map(|(idx, _)| Arc::clone(&arrays[idx]))
            .collect::<Vec<_>>();
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => StructArray(projected.into()),
        }
    }
}

impl<T: StructArrayType, Buffer: BufferType> From<arrow_array::StructArray>
    for StructArray<T, Nullable, Buffer>
where
    <T as StructArrayType>::Array<Buffer>:
        From<Vec<Arc<dyn arrow_array::Array>>> + Length + StructArrayTypeFields,
    Bitmap<Buffer>: From<NullBuffer> + FromIterator<bool>,
{
    fn from(value: arrow_array::StructArray) -> Self {
        let (fields, arrays, nulls_opt) = value.into_parts();
        // Project
        let projected = <<T as StructArrayType>::Array<Buffer> as StructArrayTypeFields>::NAMES
            .iter()
            .map(|field| {
                fields
                    .find(field)
                    .unwrap_or_else(|| panic!("expected struct array with field: {field}"))
            })
            .map(|(idx, _)| Arc::clone(&arrays[idx]))
            .collect::<Vec<_>>();
        let data = projected.into();
        match nulls_opt {
            Some(null_buffer) => StructArray(Validity {
                data,
                validity: null_buffer.into(),
            }),
            None => StructArray::<T, NonNullable, Buffer>(data).into(),
        }
    }
}

impl<T: StructArrayType, Nullable: Nullability, Buffer: BufferType>
    From<StructArray<T, Nullable, Buffer>> for arrow_array::RecordBatch
where
    arrow_array::StructArray: From<StructArray<T, Nullable, Buffer>>,
{
    fn from(value: StructArray<T, Nullable, Buffer>) -> Self {
        Self::from(arrow_array::StructArray::from(value))
    }
}

impl<T: StructArrayType, Nullable: Nullability, Buffer: BufferType> From<arrow_array::RecordBatch>
    for StructArray<T, Nullable, Buffer>
where
    Self: From<arrow_array::StructArray>,
{
    fn from(value: arrow_array::RecordBatch) -> Self {
        Self::from(arrow_array::StructArray::from(value))
    }
}

impl<T: StructArrayType, Nullable: Nullability, Buffer: BufferType> StructArray<T, Nullable, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: StructArrayTypeFields,
{
    /// Return the Arrow schema using the fields of this `StructArray`.
    #[must_use]
    pub fn schema() -> arrow_schema::Schema {
        arrow_schema::Schema::new(
            <<T as StructArrayType>::Array<Buffer> as StructArrayTypeFields>::fields(),
        )
    }
}

#[cfg(test)]
mod tests {

    use arrow_array::{cast::AsArray as _, types::UInt32Type, Array as _};

    use crate::{
        array::{union::UnionType, ArrayType, ArrayTypeOf},
        arrow::buffer::{BufferBuilder, ScalarBuffer},
        bitmap::ValidityBitmap,
        buffer::Buffer as _,
        offset::Offset,
    };

    use super::*;

    #[derive(Copy, Clone, Default, Debug, PartialEq)]
    struct Foo {
        a: u32,
    }
    struct FooArray<Buffer: BufferType> {
        a: ArrayTypeOf<u32, Buffer>,
    }
    impl ArrayType<Foo> for Foo {
        type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
            StructArray<Foo, NonNullable, Buffer>;
    }
    impl ArrayType<Foo> for Option<Foo> {
        type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
            StructArray<Foo, Nullable, Buffer>;
    }
    impl<Buffer: BufferType> Default for FooArray<Buffer>
    where
        ArrayTypeOf<u32, Buffer>: Default,
    {
        fn default() -> Self {
            Self {
                a: <ArrayTypeOf<u32, Buffer>>::default(),
            }
        }
    }
    impl<Buffer: BufferType> Extend<Foo> for FooArray<Buffer>
    where
        ArrayTypeOf<u32, Buffer>: Extend<u32>,
    {
        fn extend<I: IntoIterator<Item = Foo>>(&mut self, iter: I) {
            iter.into_iter().for_each(|Foo { a }| {
                self.a.extend(std::iter::once(a));
            });
        }
    }
    impl<Buffer: BufferType> FromIterator<Foo> for FooArray<Buffer>
    where
        ArrayTypeOf<u32, Buffer>: Default + Extend<u32>,
    {
        fn from_iter<T: IntoIterator<Item = Foo>>(iter: T) -> Self {
            let (a, _): (_, Vec<_>) = iter.into_iter().map(|Foo { a }| (a, ())).unzip();
            Self { a }
        }
    }
    struct FooArrayIter<Buffer: BufferType>
    where
        ArrayTypeOf<u32, Buffer>: IntoIterator,
    {
        a: <ArrayTypeOf<u32, Buffer> as IntoIterator>::IntoIter,
    }
    impl<Buffer: BufferType> Iterator for FooArrayIter<Buffer>
    where
        ArrayTypeOf<u32, Buffer>: IntoIterator<Item = u32>,
    {
        type Item = Foo;

        fn next(&mut self) -> Option<Self::Item> {
            self.a.next().map(|a| Foo { a })
        }
    }
    impl<Buffer: BufferType> IntoIterator for FooArray<Buffer>
    where
        ArrayTypeOf<u32, Buffer>: IntoIterator<Item = u32>,
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
        const NAMES: &'static [&'static str] = &["a"];
        fn fields() -> Fields {
            Fields::from(vec![Field::new("a", DataType::UInt32, false)])
        }
    }
    impl<Buffer: BufferType> From<FooArray<Buffer>> for Vec<Arc<dyn arrow_array::Array>>
    where
        ArrayTypeOf<u32, Buffer>: Into<<ArrayTypeOf<u32, Buffer> as crate::arrow::Array>::Array>,
    {
        fn from(value: FooArray<Buffer>) -> Self {
            vec![Arc::<
                <ArrayTypeOf<u32, Buffer> as crate::arrow::Array>::Array,
            >::new(value.a.into())]
        }
    }
    impl<Buffer: BufferType> From<Vec<Arc<dyn arrow_array::Array>>> for FooArray<Buffer>
    where
        ArrayTypeOf<u32, Buffer>: From<Arc<dyn arrow_array::Array>>,
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
            .collect::<StructArray<Foo, NonNullable, BufferBuilder>>();
        let struct_array_arrow = arrow_array::StructArray::from(struct_array);
        assert_eq!(struct_array_arrow.len(), 2);

        let struct_array_nullable = [Some(Foo { a: 1234 }), None]
            .into_iter()
            .collect::<StructArray<Foo, Nullable, BufferBuilder>>();
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
        let roundtrip: StructArray<Foo, Nullable, ScalarBuffer> =
            struct_array_arrow_nullable.into();
        assert_eq!(roundtrip.0.data.a.0, [1234, u32::default()]);
    }

    #[test]
    fn into_nullable() {
        let struct_array = [Foo { a: 1 }, Foo { a: 2 }]
            .into_iter()
            .collect::<StructArray<Foo, NonNullable, BufferBuilder>>();
        let struct_array_arrow = arrow_array::StructArray::from(struct_array);
        assert!(!StructArray::<Foo, Nullable, ScalarBuffer>::from(struct_array_arrow).any_null());
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let struct_array_nullable = [Some(Foo { a: 1234 }), None]
            .into_iter()
            .collect::<StructArray<Foo, Nullable, BufferBuilder>>();
        let struct_array_arrow_nullable = arrow_array::StructArray::from(struct_array_nullable);
        let _ = StructArray::<Foo, NonNullable, ScalarBuffer>::from(struct_array_arrow_nullable);
    }

    #[test]
    fn into() {
        let struct_array = [Foo { a: 1 }, Foo { a: 2 }]
            .into_iter()
            .collect::<StructArray<Foo, NonNullable, BufferBuilder>>();
        let struct_array_arrow = arrow_array::StructArray::from(struct_array);
        assert_eq!(
            StructArray::<Foo, NonNullable, ScalarBuffer>::from(struct_array_arrow)
                .0
                .a
                .0,
            [1, 2]
        );
        let struct_array_nullable = [Some(Foo { a: 1234 }), None]
            .into_iter()
            .collect::<StructArray<Foo, Nullable, BufferBuilder>>();
        let struct_array_arrow_nullable = arrow_array::StructArray::from(struct_array_nullable);
        assert_eq!(
            StructArray::<Foo, Nullable, ScalarBuffer>::from(struct_array_arrow_nullable)
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
        let struct_array = input.into_iter().collect::<StructArray<Foo, NonNullable>>();
        let vec = struct_array.into_iter().collect::<Vec<Foo>>();
        assert_eq!(input.as_slice(), vec.as_slice());

        let input_nullable = [Some(Foo { a: 1 }), None, Some(Foo { a: 2345 })];
        let struct_array_nullable = input_nullable
            .into_iter()
            .collect::<StructArray<Foo, Nullable>>();
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
            .collect::<StructArray<Foo<_>, NonNullable>>();
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
        assert!(!fields[0].is_nullable());
        assert_eq!(*fields[0].data_type(), arrow_schema::DataType::UInt8);

        assert_eq!(fields[1].name(), "b");
        assert!(fields[1].is_nullable());
        assert_eq!(
            *fields[1].data_type(),
            arrow_schema::DataType::List(Arc::new(Field::new(
                "item",
                arrow_schema::DataType::Int32,
                false
            )))
        );
    }

    #[test]
    #[should_panic(expected = "expected struct array with field: c")]
    #[cfg(feature = "derive")]
    fn projected() {
        #[derive(narrow_derive::ArrayType)]
        struct Foo {
            a: u32,
            b: bool,
            c: u64,
        }

        #[derive(narrow_derive::ArrayType, Debug, PartialEq)]
        struct Bar {
            b: bool,
            a: u32,
        }

        let foo_array = [
            Foo {
                a: 1,
                b: false,
                c: 2,
            },
            Foo {
                a: 2,
                b: true,
                c: 3,
            },
        ]
        .into_iter()
        .collect::<StructArray<Foo>>();

        let arrow_array = arrow_array::StructArray::from(foo_array);
        let bar_array = StructArray::<Bar>::from(arrow_array);
        assert_eq!(
            bar_array.clone().into_iter().collect::<Vec<_>>(),
            [Bar { b: false, a: 1 }, Bar { b: true, a: 2 }]
        );

        let bar_arrow_array = arrow_array::StructArray::from(bar_array);
        let _ = StructArray::<Foo>::from(bar_arrow_array);
    }
}
