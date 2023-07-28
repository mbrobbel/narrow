struct Foo {
    a: u32,
    b: bool,
    c: Option<Vec<u8>>,
}
impl narrow::array::ArrayType for Foo {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Foo,
        false,
        Buffer,
    >;
}
impl narrow::array::ArrayType<Foo> for ::std::option::Option<Foo> {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Foo,
        true,
        Buffer,
    >;
}
impl narrow::array::StructArrayType for Foo
where
    u32: narrow::array::ArrayType,
    bool: narrow::array::ArrayType,
    Option<Vec<u8>>: narrow::array::ArrayType,
{
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<Buffer>;
}
struct FooArray<Buffer: narrow::buffer::BufferType>
where
    u32: narrow::array::ArrayType,
    bool: narrow::array::ArrayType,
    Option<Vec<u8>>: narrow::array::ArrayType,
{
    a: <u32 as narrow::array::ArrayType>::Array<Buffer>,
    b: <bool as narrow::array::ArrayType>::Array<Buffer>,
    c: <Option<Vec<u8>> as narrow::array::ArrayType>::Array<Buffer>,
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::FromIterator<Foo>
for FooArray<Buffer>
where
    u32: narrow::array::ArrayType,
    bool: narrow::array::ArrayType,
    Option<Vec<u8>>: narrow::array::ArrayType,
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
    >: ::std::default::Default + ::std::iter::Extend<u32>,
    <bool as narrow::array::ArrayType>::Array<
        Buffer,
    >: ::std::default::Default + ::std::iter::Extend<bool>,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType>::Array<
        Buffer,
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
impl<Buffer: narrow::buffer::BufferType> ::std::default::Default for FooArray<Buffer>
where
    u32: narrow::array::ArrayType,
    bool: narrow::array::ArrayType,
    Option<Vec<u8>>: narrow::array::ArrayType,
    <u32 as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
    <bool as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
{
    fn default() -> Self {
        Self {
            a: ::std::default::Default::default(),
            b: ::std::default::Default::default(),
            c: ::std::default::Default::default(),
        }
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Foo> for FooArray<Buffer>
where
    u32: narrow::array::ArrayType,
    bool: narrow::array::ArrayType,
    Option<Vec<u8>>: narrow::array::ArrayType,
    <u32 as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<u32>,
    <bool as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<bool>,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<Option<Vec<u8>>>,
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
impl<Buffer: narrow::buffer::BufferType> narrow::Length for FooArray<Buffer>
where
    u32: narrow::array::ArrayType,
    <u32 as narrow::array::ArrayType>::Array<Buffer>: narrow::Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.a.len()
    }
}
