struct Foo {
    a: u32,
    b: bool,
    c: Option<Vec<u8>>,
}
impl narrow::array::ArrayType for Foo {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo, false, Buffer>;
}
impl narrow::array::ArrayType<Foo> for ::std::option::Option<Foo> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo, true, Buffer>;
}
impl narrow::array::StructArrayType for Foo {
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<Buffer>;
}
impl<Buffer: narrow::buffer::BufferType> narrow::arrow::StructArrayTypeFields
for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
    <bool as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
{
    fn fields() -> ::arrow_schema::Fields {
        ::arrow_schema::Fields::from([
            ::std::sync::Arc::new(
                <<u32 as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("a"),
            ),
            ::std::sync::Arc::new(
                <<bool as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("b"),
            ),
            ::std::sync::Arc::new(
                <<Option<
                    Vec<u8>,
                > as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("c"),
            ),
        ])
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::convert::From<FooArray<Buffer>>
for ::std::vec::Vec<::std::sync::Arc<dyn ::arrow_array::Array>>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<u32 as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
    <bool as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<bool as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<Option<
            Vec<u8>,
        > as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
{
    fn from(value: FooArray<Buffer>) -> Self {
        <[_]>::into_vec(
            #[rustc_box]
            ::alloc::boxed::Box::new([
                ::std::sync::Arc::<
                    <<u32 as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.a.into()),
                ::std::sync::Arc::<
                    <<bool as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.b.into()),
                ::std::sync::Arc::<
                    <<Option<
                        Vec<u8>,
                    > as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.c.into()),
            ]),
        )
    }
}
struct FooArray<Buffer: narrow::buffer::BufferType> {
    a: <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
    b: <bool as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
    c: <Option<
        Vec<u8>,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
}
impl<Buffer: narrow::buffer::BufferType> ::std::default::Default for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <bool as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
{
    fn default() -> Self {
        Self {
            a: ::std::default::Default::default(),
            b: ::std::default::Default::default(),
            c: ::std::default::Default::default(),
        }
    }
}
impl<Buffer: narrow::buffer::BufferType> narrow::Length for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
    <bool as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
{
    fn len(&self) -> usize {
        self.a.len()
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Foo> for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<u32>,
    <bool as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<bool>,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<Option<Vec<u8>>>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Foo>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|Foo { a, b, c }| {
                self.a.extend(::std::iter::once(a));
                self.b.extend(::std::iter::once(b));
                self.c.extend(::std::iter::once(c));
            });
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::FromIterator<Foo>
for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<u32>,
    <bool as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<bool>,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<Option<Vec<u8>>>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Foo>>(iter: _I) -> Self {
        let (a, (b, (c, ()))) = iter
            .into_iter()
            .map(|Foo { a, b, c }| (a, (b, (c, ()))))
            .unzip();
        Self { a, b, c }
    }
}
