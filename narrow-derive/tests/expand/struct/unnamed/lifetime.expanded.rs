struct Foo<'a, T>(&'a T);
impl<'a, T: narrow::array::ArrayType> narrow::array::ArrayType for Foo<'a, T> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<'a, T>, false, Buffer>;
}
impl<'a, T: narrow::array::ArrayType> narrow::array::ArrayType<Foo<'a, T>>
for ::std::option::Option<Foo<'a, T>> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<'a, T>, true, Buffer>;
}
impl<'a, T: narrow::array::ArrayType> narrow::array::StructArrayType for Foo<'a, T> {
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<'a, T, Buffer>;
}
impl<
    'a,
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> narrow::arrow::StructArrayTypeFields for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
{
    fn fields() -> ::arrow_schema::Fields {
        ::arrow_schema::Fields::from([
            ::std::sync::Arc::new(
                <<&'a T as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("_0"),
            ),
        ])
    }
}
impl<
    'a,
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::convert::From<FooArray<'a, T, Buffer>>
for ::std::vec::Vec<::std::sync::Arc<dyn ::arrow_array::Array>>
where
    <&'a T as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<&'a T as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
{
    fn from(value: FooArray<'a, T, Buffer>) -> Self {
        <[_]>::into_vec(
            #[rustc_box]
            ::alloc::boxed::Box::new([
                ::std::sync::Arc::<
                    <<&'a T as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.0.into()),
            ]),
        )
    }
}
struct FooArray<'a, T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType>(
    <&'a T as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
);
impl<
    'a,
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<'a, T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType> narrow::Length
for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
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
    <&'a T as narrow::array::ArrayType>::Array<
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
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Foo<'a, T>> for FooArray<'a, T, Buffer>
where
    <&'a T as narrow::array::ArrayType>::Array<
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
