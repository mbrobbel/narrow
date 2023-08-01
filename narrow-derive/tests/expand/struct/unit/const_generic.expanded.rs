pub struct Foo<const N: usize>;
/// Safety:
/// - This is a unit struct.
unsafe impl<const N: usize> narrow::array::Unit for Foo<N> {}
impl<const N: usize> narrow::array::ArrayType for Foo<N> {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Foo<N>,
        false,
        Buffer,
    >;
}
impl<const N: usize> narrow::array::ArrayType<Foo<N>> for ::std::option::Option<Foo<N>> {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Foo<N>,
        true,
        Buffer,
    >;
}
impl<const N: usize> narrow::array::StructArrayType for Foo<N> {
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<N, Buffer>;
}
pub struct FooArray<const N: usize, Buffer: narrow::buffer::BufferType>(
    narrow::array::NullArray<Foo<N>, false, Buffer>,
);
impl<const N: usize, Buffer: narrow::buffer::BufferType> ::std::default::Default
for FooArray<N, Buffer> {
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<const N: usize, Buffer: narrow::buffer::BufferType> narrow::Length
for FooArray<N, Buffer> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<const N: usize, Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Foo<N>>
for FooArray<N, Buffer> {
    fn extend<_I: ::std::iter::IntoIterator<Item = Foo<N>>>(&mut self, iter: _I) {
        self.0.extend(iter)
    }
}
impl<
    const N: usize,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Foo<N>> for FooArray<N, Buffer> {
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Foo<N>>>(iter: _I) -> Self {
        Self(iter.into_iter().collect())
    }
}
