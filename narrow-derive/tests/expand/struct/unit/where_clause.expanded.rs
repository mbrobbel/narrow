pub(super) struct Foo<const N: bool = false>
where
    Self: Sized,
    (): From<Self>;
/// Safety:
/// - This is a unit struct.
unsafe impl<const N: bool> narrow::array::Unit for Foo<N>
where
    Self: Sized,
    (): From<Self>,
{
    type Item = Self;
}
impl<const N: bool> narrow::array::ArrayType<Foo<N>> for Foo<N>
where
    Self: Sized,
    (): From<Self>,
{
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<N>, narrow::NonNullable, Buffer>;
}
impl<const N: bool> narrow::array::ArrayType<Foo<N>> for ::std::option::Option<Foo<N>>
where
    Self: Sized,
    (): From<Self>,
{
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<N>, narrow::Nullable, Buffer>;
}
impl<const N: bool> narrow::array::StructArrayType for Foo<N>
where
    Self: Sized,
    (): From<Self>,
{
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<N, Buffer>;
}
pub(super) struct FooArray<const N: bool, Buffer: narrow::buffer::BufferType>(
    pub(super) narrow::array::NullArray<Foo<N>, narrow::NonNullable, Buffer>,
)
where
    Foo<N>: Sized,
    (): From<Foo<N>>;
impl<const N: bool, Buffer: narrow::buffer::BufferType> ::std::clone::Clone
for FooArray<N, Buffer>
where
    Foo<N>: Sized,
    (): From<Foo<N>>,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<const N: bool, Buffer: narrow::buffer::BufferType> ::std::default::Default
for FooArray<N, Buffer>
where
    Foo<N>: Sized,
    (): From<Foo<N>>,
{
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<const N: bool, Buffer: narrow::buffer::BufferType> narrow::Length
for FooArray<N, Buffer>
where
    Foo<N>: Sized,
    (): From<Foo<N>>,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<const N: bool, Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Foo<N>>
for FooArray<N, Buffer>
where
    Self: Sized,
    (): From<Self>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Foo<N>>>(&mut self, iter: _I) {
        self.0.extend(iter)
    }
}
impl<const N: bool, Buffer: narrow::buffer::BufferType> ::std::iter::FromIterator<Foo<N>>
for FooArray<N, Buffer>
where
    Self: Sized,
    (): From<Self>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Foo<N>>>(iter: _I) -> Self {
        Self(iter.into_iter().collect())
    }
}
pub(super) struct FooArrayIter<const N: bool, Buffer: narrow::buffer::BufferType>(
    pub(super) <narrow::array::NullArray<
        Foo<N>,
        narrow::NonNullable,
        Buffer,
    > as IntoIterator>::IntoIter,
)
where
    Self: Sized,
    (): From<Self>,
    narrow::array::NullArray<
        Foo<N>,
        narrow::NonNullable,
        Buffer,
    >: ::std::iter::IntoIterator<Item = Foo<N>>;
impl<const N: bool, Buffer: narrow::buffer::BufferType> ::std::iter::Iterator
for FooArrayIter<N, Buffer>
where
    Self: Sized,
    (): From<Self>,
{
    type Item = Foo<N>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
impl<const N: bool, Buffer: narrow::buffer::BufferType> ::std::iter::IntoIterator
for FooArray<N, Buffer>
where
    Self: Sized,
    (): From<Self>,
{
    type Item = Foo<N>;
    type IntoIter = FooArrayIter<N, Buffer>;
    fn into_iter(self) -> Self::IntoIter {
        FooArrayIter(self.0.into_iter())
    }
}
