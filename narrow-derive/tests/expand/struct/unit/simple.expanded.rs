struct Foo;
/// Safety:
/// - This is a unit struct.
unsafe impl narrow::array::Unit for Foo {
    type Item = Self;
}
impl narrow::array::ArrayType for Foo {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo, false, Buffer>;
}
impl narrow::array::ArrayType<Foo> for ::std::option::Option<Foo> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo, true, Buffer>;
}
impl narrow::array::StructArrayType for Foo {
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<Buffer>;
}
impl<Buffer: narrow::buffer::BufferType> narrow::arrow::StructArrayTypeFields
for FooArray<Buffer> {
    fn fields() -> ::arrow_schema::Fields {
        ::arrow_schema::Fields::from([])
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::convert::From<FooArray<Buffer>>
for ::std::vec::Vec<::std::sync::Arc<dyn ::arrow_array::Array>> {
    fn from(value: FooArray<Buffer>) -> Self {
        ::alloc::vec::Vec::new()
    }
}
struct FooArray<Buffer: narrow::buffer::BufferType>(
    narrow::array::NullArray<Foo, false, Buffer>,
);
impl<Buffer: narrow::buffer::BufferType> ::std::default::Default for FooArray<Buffer> {
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<Buffer: narrow::buffer::BufferType> narrow::Length for FooArray<Buffer> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Foo> for FooArray<Buffer> {
    fn extend<_I: ::std::iter::IntoIterator<Item = Foo>>(&mut self, iter: _I) {
        self.0.extend(iter)
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::FromIterator<Foo>
for FooArray<Buffer> {
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Foo>>(iter: _I) -> Self {
        Self(iter.into_iter().collect())
    }
}
