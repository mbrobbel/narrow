struct Bar<T> {
    a: u32,
    b: Option<bool>,
    c: Option<T>,
}
impl<T: narrow::array::ArrayType> narrow::array::ArrayType for Bar<T> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar<T>, false, Buffer>;
}
impl<T: narrow::array::ArrayType> narrow::array::ArrayType<Bar<T>>
for ::std::option::Option<Bar<T>> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar<T>, true, Buffer>;
}
impl<T: narrow::array::ArrayType> narrow::array::StructArrayType for Bar<T> {
    type Array<Buffer: narrow::buffer::BufferType> = BarArray<T, Buffer>;
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> narrow::arrow::StructArrayTypeFields for BarArray<T, Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
    <Option<
        bool,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
    <Option<
        T,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
{
    fn fields() -> ::arrow_schema::Fields {
        ::arrow_schema::Fields::from([
            ::std::sync::Arc::new(
                <<u32 as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("a"),
            ),
            ::std::sync::Arc::new(
                <<Option<
                    bool,
                > as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("b"),
            ),
            ::std::sync::Arc::new(
                <<Option<
                    T,
                > as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("c"),
            ),
        ])
    }
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::convert::From<BarArray<T, Buffer>>
for ::std::vec::Vec<::std::sync::Arc<dyn ::arrow_array::Array>>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<u32 as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
    <Option<
        bool,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<Option<
            bool,
        > as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
    <Option<
        T,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<Option<
            T,
        > as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
{
    fn from(value: BarArray<T, Buffer>) -> Self {
        <[_]>::into_vec(
            #[rustc_box]
            ::alloc::boxed::Box::new([
                ::std::sync::Arc::<
                    <<u32 as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.a.into()),
                ::std::sync::Arc::<
                    <<Option<
                        bool,
                    > as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.b.into()),
                ::std::sync::Arc::<
                    <<Option<
                        T,
                    > as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.c.into()),
            ]),
        )
    }
}
struct BarArray<T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType> {
    a: <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
    b: <Option<
        bool,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
    c: <Option<
        T,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for BarArray<T, Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <Option<
        bool,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <Option<
        T,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
{
    fn default() -> Self {
        Self {
            a: ::std::default::Default::default(),
            b: ::std::default::Default::default(),
            c: ::std::default::Default::default(),
        }
    }
}
impl<T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType> narrow::Length
for BarArray<T, Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
    <Option<
        bool,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
    <Option<
        T,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
{
    fn len(&self) -> usize {
        self.a.len()
    }
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Extend<Bar<T>> for BarArray<T, Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<u32>,
    <Option<
        bool,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<Option<bool>>,
    <Option<
        T,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<Option<T>>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Bar<T>>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|Bar { a, b, c }| {
                self.a.extend(::std::iter::once(a));
                self.b.extend(::std::iter::once(b));
                self.c.extend(::std::iter::once(c));
            });
    }
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Bar<T>> for BarArray<T, Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<u32>,
    <Option<
        bool,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<Option<bool>>,
    <Option<
        T,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<Option<T>>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Bar<T>>>(iter: _I) -> Self {
        let (a, (b, (c, ()))) = iter
            .into_iter()
            .map(|Bar { a, b, c }| (a, (b, (c, ()))))
            .unzip();
        Self { a, b, c }
    }
}
