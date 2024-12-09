struct Foo
where
    Self: Debug;
/// Safety:
/// - This is a unit struct.
unsafe impl narrow::array::Unit for Foo
where
    Self: Debug,
{
    type Item = Self;
}
impl narrow::array::ArrayType<Foo> for Foo
where
    Self: Debug,
{
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo, narrow::NonNullable, Buffer>;
}
impl narrow::array::ArrayType<Foo> for ::std::option::Option<Foo>
where
    Self: Debug,
{
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo, narrow::Nullable, Buffer>;
}
impl narrow::array::StructArrayType for Foo
where
    Self: Debug,
{
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<Buffer>;
}
struct FooArray<Buffer: narrow::buffer::BufferType>(
    narrow::array::NullArray<Foo, narrow::NonNullable, Buffer>,
)
where
    Foo: Debug;
impl<Buffer: narrow::buffer::BufferType> ::std::clone::Clone for FooArray<Buffer>
where
    Foo: Debug,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::default::Default for FooArray<Buffer>
where
    Foo: Debug,
{
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<Buffer: narrow::buffer::BufferType> narrow::Length for FooArray<Buffer>
where
    Foo: Debug,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Foo> for FooArray<Buffer>
where
    Self: Debug,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Foo>>(&mut self, iter: _I) {
        self.0.extend(iter)
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::FromIterator<Foo>
for FooArray<Buffer>
where
    Self: Debug,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Foo>>(iter: _I) -> Self {
        Self(iter.into_iter().collect())
    }
}
struct FooArrayIter<Buffer: narrow::buffer::BufferType>(
    <narrow::array::NullArray<
        Foo,
        narrow::NonNullable,
        Buffer,
    > as IntoIterator>::IntoIter,
)
where
    Self: Debug,
    narrow::array::NullArray<
        Foo,
        narrow::NonNullable,
        Buffer,
    >: ::std::iter::IntoIterator<Item = Foo>;
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Iterator for FooArrayIter<Buffer>
where
    Self: Debug,
{
    type Item = Foo;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::IntoIterator for FooArray<Buffer>
where
    Self: Debug,
{
    type Item = Foo;
    type IntoIter = FooArrayIter<Buffer>;
    fn into_iter(self) -> Self::IntoIter {
        FooArrayIter(self.0.into_iter())
    }
}
