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
impl narrow::array::ArrayType for Foo
where
    Self: Debug,
{
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Foo,
        false,
        Buffer,
    >;
}
impl narrow::array::ArrayType<Foo> for ::std::option::Option<Foo>
where
    Self: Debug,
{
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Foo,
        true,
        Buffer,
    >;
}
impl narrow::array::StructArrayType for Foo
where
    Self: Debug,
{
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<Buffer>;
}
struct FooArray<Buffer: narrow::buffer::BufferType>(
    narrow::array::NullArray<Foo, false, Buffer>,
)
where
    Foo: Debug;
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
