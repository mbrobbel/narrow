struct Foo {
    a: u32,
    b: bool,
    c: Option<Vec<u8>>,
}
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
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::union::UnionType,
    > = FooArray<Buffer>;
}
struct FooArray<Buffer: narrow::buffer::BufferType> {
    a: <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
    b: <bool as narrow::array::ArrayType<
        bool,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
    c: <Option<
        Vec<u8>,
    > as narrow::array::ArrayType<
        Vec<u8>,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
}
impl<Buffer: narrow::buffer::BufferType> ::std::clone::Clone for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
    <bool as narrow::array::ArrayType<
        bool,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType<
        Vec<u8>,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self {
            a: self.a.clone(),
            b: self.b.clone(),
            c: self.c.clone(),
        }
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::default::Default for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <bool as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType<
        Vec<u8>,
    >>::Array<
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
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
    <bool as narrow::array::ArrayType<
        bool,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType<
        Vec<u8>,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
{
    fn len(&self) -> usize {
        self.a.len()
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
    <bool as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<bool>,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType<
        Vec<u8>,
    >>::Array<
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
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<u32>,
    <bool as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<bool>,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType<
        Vec<u8>,
    >>::Array<
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
struct FooArrayIter<Buffer: narrow::buffer::BufferType>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u32>,
    <bool as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = bool>,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType<
        Vec<u8>,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = Option<Vec<u8>>>,
{
    a: <<u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
    b: <<bool as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
    c: <<Option<
        Vec<u8>,
    > as narrow::array::ArrayType<
        Vec<u8>,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Iterator for FooArrayIter<Buffer>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u32>,
    <bool as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = bool>,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType<
        Vec<u8>,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = Option<Vec<u8>>>,
{
    type Item = Foo;
    fn next(&mut self) -> Option<Self::Item> {
        self.a
            .next()
            .map(|a| {
                Foo {
                    a,
                    b: self.b.next().unwrap(),
                    c: self.c.next().unwrap(),
                }
            })
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::IntoIterator for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u32>,
    <bool as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = bool>,
    <Option<
        Vec<u8>,
    > as narrow::array::ArrayType<
        Vec<u8>,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = Option<Vec<u8>>>,
{
    type Item = Foo;
    type IntoIter = FooArrayIter<Buffer>;
    fn into_iter(self) -> Self::IntoIter {
        FooArrayIter {
            a: self.a.into_iter(),
            b: self.b.into_iter(),
            c: self.c.into_iter(),
        }
    }
}
