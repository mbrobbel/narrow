struct Foo<T: Sized>(T, u32);
impl<T: Sized + narrow::array::ArrayType<T>> narrow::array::ArrayType<Foo<T>>
for Foo<T> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<T>, false, Buffer>;
}
impl<T: Sized + narrow::array::ArrayType<T>> narrow::array::ArrayType<Foo<T>>
for ::std::option::Option<Foo<T>> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<T>, true, Buffer>;
}
impl<T: Sized + narrow::array::ArrayType<T>> narrow::array::StructArrayType for Foo<T> {
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<T, Buffer>;
}
struct FooArray<
    T: Sized + narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
>(
    <T as narrow::array::ArrayType<
        T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
);
impl<
    T: Sized + narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for FooArray<T, Buffer>
where
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default(), ::std::default::Default::default())
    }
}
impl<
    T: Sized + narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> narrow::Length for FooArray<T, Buffer>
where
    <T as narrow::array::ArrayType<
        T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<
    T: Sized + narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Extend<Foo<T>> for FooArray<T, Buffer>
where
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<T>,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<u32>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Foo<T>>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|Foo(_0, _1)| {
                self.0.extend(::std::iter::once(_0));
                self.1.extend(::std::iter::once(_1));
            });
    }
}
impl<
    T: Sized + narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Foo<T>> for FooArray<T, Buffer>
where
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<T>,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<u32>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Foo<T>>>(iter: _I) -> Self {
        let (_0, (_1, ())) = iter.into_iter().map(|Foo(_0, _1)| (_0, (_1, ()))).unzip();
        Self(_0, _1)
    }
}
struct FooArrayIter<
    T: Sized + narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
>(
    <<T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
    <<u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
)
where
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = T>,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u32>;
impl<
    T: Sized + narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Iterator for FooArrayIter<T, Buffer>
where
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = T>,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u32>,
{
    type Item = Foo<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|first| { Foo(first, self.1.next().unwrap()) })
    }
}
impl<
    T: Sized + narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::IntoIterator for FooArray<T, Buffer>
where
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = T>,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u32>,
{
    type Item = Foo<T>;
    type IntoIter = FooArrayIter<T, Buffer>;
    fn into_iter(self) -> Self::IntoIter {
        FooArrayIter(self.0.into_iter(), self.1.into_iter())
    }
}
