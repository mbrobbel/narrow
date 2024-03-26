struct Foo(u32);
impl narrow::array::ArrayType<Foo> for Foo {
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
struct FooArray<Buffer: narrow::buffer::BufferType>(
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
);
impl<Buffer: narrow::buffer::BufferType> ::std::default::Default for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
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
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Foo> for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
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
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
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
impl narrow::array::ArrayType<Bar> for Bar {
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
struct BarArray<Buffer: narrow::buffer::BufferType>(
    <Foo as narrow::array::ArrayType<
        Foo,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
);
impl<Buffer: narrow::buffer::BufferType> ::std::default::Default for BarArray<Buffer>
where
    <Foo as narrow::array::ArrayType<
        Foo,
    >>::Array<
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
    <Foo as narrow::array::ArrayType<
        Foo,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Bar> for BarArray<Buffer>
where
    <Foo as narrow::array::ArrayType<
        Foo,
    >>::Array<
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
    <Foo as narrow::array::ArrayType<
        Foo,
    >>::Array<
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
