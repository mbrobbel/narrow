struct Foo<T: Sized>(T);
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
{
    fn default() -> Self {
        Self(::std::default::Default::default())
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
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Foo<T>>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|Foo(_0)| {
                self.0.extend(::std::iter::once(_0));
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
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Foo<T>>>(iter: _I) -> Self {
        let (_0, ()) = iter.into_iter().map(|Foo(_0)| (_0, ())).unzip();
        Self(_0)
    }
}
