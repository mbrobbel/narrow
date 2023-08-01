struct Foo<'a, T>(&'a T);
impl<'a, T: narrow::array::ArrayType> narrow::array::ArrayType for Foo<'a, T> {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Foo<'a, T>,
        false,
        Buffer,
    >;
}
impl<'a, T: narrow::array::ArrayType> narrow::array::ArrayType<Foo<'a, T>>
for ::std::option::Option<Foo<'a, T>> {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Foo<'a, T>,
        true,
        Buffer,
    >;
}
impl<'a, T: narrow::array::ArrayType> narrow::array::StructArrayType for Foo<'a, T> {
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<'a, T, Buffer>;
}
struct FooArray<'a, T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType>(
    <&'a T as narrow::array::ArrayType>::Array<Buffer>,
);
impl<
    'a,
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<'a, T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType> narrow::Length
for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType>::Array<Buffer>: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<
    'a,
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Extend<Foo<'a, T>> for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<&'a T>,
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
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Foo<'a, T>> for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType>::Array<
        Buffer,
    >: ::std::default::Default + ::std::iter::Extend<&'a T>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Foo<'a, T>>>(iter: _I) -> Self {
        let (_0, ()) = iter.into_iter().map(|Foo(_0)| (_0, ())).unzip();
        Self(_0)
    }
}
