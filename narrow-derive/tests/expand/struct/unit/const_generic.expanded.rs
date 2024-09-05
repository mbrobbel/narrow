pub struct Foo<const N: usize>;
/// Safety:
/// - This is a unit struct.
unsafe impl<const N: usize> narrow::array::Unit for Foo<N> {
    type Item = Self;
}
impl<const N: usize> narrow::array::ArrayType<Foo<N>> for Foo<N> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<N>, false, Buffer>;
}
impl<const N: usize> narrow::array::ArrayType<Foo<N>> for ::std::option::Option<Foo<N>> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<N>, true, Buffer>;
}
impl<const N: usize> narrow::array::StructArrayType for Foo<N> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::union::UnionType,
    > = FooArray<N, Buffer>;
}
pub struct FooArray<const N: usize, Buffer: narrow::buffer::BufferType>(
    pub narrow::array::NullArray<Foo<N>, false, Buffer>,
);
impl<const N: usize, Buffer: narrow::buffer::BufferType> ::std::clone::Clone
for FooArray<N, Buffer> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
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
pub struct FooArrayIter<const N: usize, Buffer: narrow::buffer::BufferType>(
    pub <narrow::array::NullArray<Foo<N>, false, Buffer> as IntoIterator>::IntoIter,
)
where
    narrow::array::NullArray<
        Foo<N>,
        false,
        Buffer,
    >: ::std::iter::IntoIterator<Item = Foo<N>>;
impl<const N: usize, Buffer: narrow::buffer::BufferType> ::std::iter::Iterator
for FooArrayIter<N, Buffer> {
    type Item = Foo<N>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
impl<const N: usize, Buffer: narrow::buffer::BufferType> ::std::iter::IntoIterator
for FooArray<N, Buffer> {
    type Item = Foo<N>;
    type IntoIter = FooArrayIter<N, Buffer>;
    fn into_iter(self) -> Self::IntoIter {
        FooArrayIter(self.0.into_iter())
    }
}
