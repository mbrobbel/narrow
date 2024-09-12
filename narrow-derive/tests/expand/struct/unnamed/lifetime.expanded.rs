struct Foo<'a, T>(&'a T);
impl<'a, T: narrow::array::ArrayType<T>> narrow::array::ArrayType<Foo<'a, T>>
for Foo<'a, T> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<'a, T>, false, Buffer>;
}
impl<'a, T: narrow::array::ArrayType<T>> narrow::array::ArrayType<Foo<'a, T>>
for ::std::option::Option<Foo<'a, T>> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<'a, T>, true, Buffer>;
}
impl<'a, T: narrow::array::ArrayType<T>> narrow::array::StructArrayType for Foo<'a, T> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::union::UnionType,
    > = FooArray<'a, T, Buffer>;
}
struct FooArray<'a, T: narrow::array::ArrayType<T>, Buffer: narrow::buffer::BufferType>(
    <&'a T as narrow::array::ArrayType<
        &'a T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
);
impl<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::clone::Clone for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType<
        &'a T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType<
        &'a T,
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
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> narrow::Length for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType<
        &'a T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Extend<Foo<'a, T>> for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType<
        &'a T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<&'a T>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Foo<'a, T>>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|Foo(_0)| {
                self.0.extend(::std::iter::once(_0));
            });
    }
}
impl<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Foo<'a, T>> for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType<
        &'a T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<&'a T>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Foo<'a, T>>>(iter: _I) -> Self {
        let (_0, ()) = iter.into_iter().map(|Foo(_0)| (_0, ())).unzip();
        Self(_0)
    }
}
struct FooArrayIter<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
>(
    <<&'a T as narrow::array::ArrayType<
        &'a T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
)
where
    <&'a T as narrow::array::ArrayType<
        &'a T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = &'a T>;
impl<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Iterator for FooArrayIter<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType<
        &'a T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = &'a T>,
{
    type Item = Foo<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|first| { Foo(first) })
    }
}
impl<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::IntoIterator for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType<
        &'a T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = &'a T>,
{
    type Item = Foo<'a, T>;
    type IntoIter = FooArrayIter<'a, T, Buffer>;
    fn into_iter(self) -> Self::IntoIter {
        FooArrayIter(self.0.into_iter())
    }
}
