struct Foo<'a, T>
where
    T: Copy,
{
    a: &'a T,
}
impl<'a, T: narrow::array::ArrayType> narrow::array::ArrayType for Foo<'a, T>
where
    T: Copy,
{
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Foo<'a, T>,
        false,
        Buffer,
    >;
}
impl<'a, T: narrow::array::ArrayType> narrow::array::ArrayType<Foo<'a, T>>
for ::std::option::Option<Foo<'a, T>>
where
    T: Copy,
{
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Foo<'a, T>,
        true,
        Buffer,
    >;
}
impl<'a, T: narrow::array::ArrayType> narrow::array::StructArrayType for Foo<'a, T>
where
    T: Copy,
{
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<'a, T, Buffer>;
}
struct FooArray<'a, T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType>
where
    T: Copy,
{
    a: <&'a T as narrow::array::ArrayType>::Array<Buffer>,
}
impl<
    'a,
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for FooArray<'a, T, Buffer>
where
    T: Copy,
    <&'a T as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
{
    fn default() -> Self {
        Self {
            a: ::std::default::Default::default(),
        }
    }
}
impl<'a, T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType> narrow::Length
for FooArray<'a, T, Buffer>
where
    T: Copy,
    <&'a T as narrow::array::ArrayType>::Array<Buffer>: narrow::Length,
{
    fn len(&self) -> usize {
        self.a.len()
    }
}
impl<
    'a,
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Extend<Foo<'a, T>> for FooArray<'a, T, Buffer>
where
    T: Copy,
    <&'a T as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<&'a T>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Foo<'a, T>>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|Foo { a }| {
                self.a.extend(::std::iter::once(a));
            });
    }
}
impl<
    'a,
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Foo<'a, T>> for FooArray<'a, T, Buffer>
where
    T: Copy,
    <&'a T as narrow::array::ArrayType>::Array<
        Buffer,
    >: ::std::default::Default + ::std::iter::Extend<&'a T>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Foo<'a, T>>>(iter: _I) -> Self {
        let (a, ()) = iter.into_iter().map(|Foo { a }| (a, ())).unzip();
        Self { a }
    }
}
