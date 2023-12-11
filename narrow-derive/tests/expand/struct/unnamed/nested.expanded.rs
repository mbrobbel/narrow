struct Foo(u32);
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
{
    fn fields() -> ::arrow_schema::Fields {
        ::arrow_schema::Fields::from([
            ::std::sync::Arc::new(
                <<u32 as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("_0"),
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
                >::new(value.0.into()),
            ]),
        )
    }
}
struct FooArray<Buffer: narrow::buffer::BufferType>(
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
);
impl<Buffer: narrow::buffer::BufferType> ::std::default::Default for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<Buffer: narrow::buffer::BufferType> narrow::Length for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Foo> for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<u32>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Foo>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|Foo(_0)| {
                self.0.extend(::std::iter::once(_0));
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
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Foo>>(iter: _I) -> Self {
        let (_0, ()) = iter.into_iter().map(|Foo(_0)| (_0, ())).unzip();
        Self(_0)
    }
}
struct Bar(Foo);
impl narrow::array::ArrayType for Bar {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar, false, Buffer>;
}
impl narrow::array::ArrayType<Bar> for ::std::option::Option<Bar> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar, true, Buffer>;
}
impl narrow::array::StructArrayType for Bar {
    type Array<Buffer: narrow::buffer::BufferType> = BarArray<Buffer>;
}
impl<Buffer: narrow::buffer::BufferType> narrow::arrow::StructArrayTypeFields
for BarArray<Buffer>
where
    <Foo as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
{
    fn fields() -> ::arrow_schema::Fields {
        ::arrow_schema::Fields::from([
            ::std::sync::Arc::new(
                <<Foo as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("_0"),
            ),
        ])
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::convert::From<BarArray<Buffer>>
for ::std::vec::Vec<::std::sync::Arc<dyn ::arrow_array::Array>>
where
    <Foo as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<Foo as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
{
    fn from(value: BarArray<Buffer>) -> Self {
        <[_]>::into_vec(
            #[rustc_box]
            ::alloc::boxed::Box::new([
                ::std::sync::Arc::<
                    <<Foo as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.0.into()),
            ]),
        )
    }
}
struct BarArray<Buffer: narrow::buffer::BufferType>(
    <Foo as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
);
impl<Buffer: narrow::buffer::BufferType> ::std::default::Default for BarArray<Buffer>
where
    <Foo as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<Buffer: narrow::buffer::BufferType> narrow::Length for BarArray<Buffer>
where
    <Foo as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Bar> for BarArray<Buffer>
where
    <Foo as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<Foo>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Bar>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|Bar(_0)| {
                self.0.extend(::std::iter::once(_0));
            });
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::FromIterator<Bar>
for BarArray<Buffer>
where
    <Foo as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<Foo>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Bar>>(iter: _I) -> Self {
        let (_0, ()) = iter.into_iter().map(|Bar(_0)| (_0, ())).unzip();
        Self(_0)
    }
}
