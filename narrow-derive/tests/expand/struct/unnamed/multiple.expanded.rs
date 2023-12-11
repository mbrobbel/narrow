struct Bar(u8, u16, u32, u64);
impl narrow::array::ArrayType for Bar {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar, false, Buffer>;
}
impl narrow::array::ArrayType<Bar> for ::std::option::Option<Bar> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar, true, Buffer>;
}
impl narrow::array::StructArrayType for Bar {
    type Array<Buffer: narrow::buffer::BufferType> = BarArray<Buffer>;
}
impl<Buffer: narrow::buffer::BufferType> narrow::arrow::StructArrayTypeFields
for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
    <u16 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
    <u64 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
{
    fn fields() -> ::arrow_schema::Fields {
        ::arrow_schema::Fields::from([
            ::std::sync::Arc::new(
                <<u8 as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("_0"),
            ),
            ::std::sync::Arc::new(
                <<u16 as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("_1"),
            ),
            ::std::sync::Arc::new(
                <<u32 as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("_2"),
            ),
            ::std::sync::Arc::new(
                <<u64 as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("_3"),
            ),
        ])
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::convert::From<BarArray<Buffer>>
for ::std::vec::Vec<::std::sync::Arc<dyn ::arrow_array::Array>>
where
    <u8 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<u8 as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
    <u16 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<u16 as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
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
    <u64 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<u64 as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
{
    fn from(value: BarArray<Buffer>) -> Self {
        <[_]>::into_vec(
            #[rustc_box]
            ::alloc::boxed::Box::new([
                ::std::sync::Arc::<
                    <<u8 as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.0.into()),
                ::std::sync::Arc::<
                    <<u16 as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.1.into()),
                ::std::sync::Arc::<
                    <<u32 as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.2.into()),
                ::std::sync::Arc::<
                    <<u64 as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.3.into()),
            ]),
        )
    }
}
struct BarArray<Buffer: narrow::buffer::BufferType>(
    <u8 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
    <u16 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
    <u64 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
);
impl<Buffer: narrow::buffer::BufferType> ::std::default::Default for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <u16 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <u64 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
{
    fn default() -> Self {
        Self(
            ::std::default::Default::default(),
            ::std::default::Default::default(),
            ::std::default::Default::default(),
            ::std::default::Default::default(),
        )
    }
}
impl<Buffer: narrow::buffer::BufferType> narrow::Length for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
    <u16 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
    <u64 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Bar> for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<u8>,
    <u16 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<u16>,
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<u32>,
    <u64 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<u64>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Bar>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|Bar(_0, _1, _2, _3)| {
                self.0.extend(::std::iter::once(_0));
                self.1.extend(::std::iter::once(_1));
                self.2.extend(::std::iter::once(_2));
                self.3.extend(::std::iter::once(_3));
            });
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::FromIterator<Bar>
for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<u8>,
    <u16 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<u16>,
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<u32>,
    <u64 as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<u64>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Bar>>(iter: _I) -> Self {
        let (_0, (_1, (_2, (_3, ())))) = iter
            .into_iter()
            .map(|Bar(_0, _1, _2, _3)| (_0, (_1, (_2, (_3, ())))))
            .unzip();
        Self(_0, _1, _2, _3)
    }
}
