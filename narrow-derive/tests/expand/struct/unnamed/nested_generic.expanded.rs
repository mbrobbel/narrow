struct Foo<T>(
    T,
)
where
    T: Copy;
impl<T: narrow::array::ArrayType> narrow::array::ArrayType for Foo<T>
where
    T: Copy,
{
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<T>, false, Buffer>;
}
impl<T: narrow::array::ArrayType> narrow::array::ArrayType<Foo<T>>
for ::std::option::Option<Foo<T>>
where
    T: Copy,
{
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<T>, true, Buffer>;
}
impl<T: narrow::array::ArrayType> narrow::array::StructArrayType for Foo<T>
where
    T: Copy,
{
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<T, Buffer>;
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> narrow::arrow::StructArrayTypeFields for FooArray<T, Buffer>
where
    T: Copy,
    <T as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
{
    fn fields() -> ::arrow_schema::Fields {
        ::arrow_schema::Fields::from([
            ::std::sync::Arc::new(
                <<T as ::narrow::array::ArrayType>::Array<
                    Buffer,
                    narrow::offset::NA,
                    narrow::array::union::NA,
                > as narrow::arrow::ArrowArray>::as_field("_0"),
            ),
        ])
    }
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::convert::From<FooArray<T, Buffer>>
for ::std::vec::Vec<::std::sync::Arc<dyn ::arrow_array::Array>>
where
    T: Copy,
    <T as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<T as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
{
    fn from(value: FooArray<T, Buffer>) -> Self {
        <[_]>::into_vec(
            #[rustc_box]
            ::alloc::boxed::Box::new([
                ::std::sync::Arc::<
                    <<T as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.0.into()),
            ]),
        )
    }
}
struct FooArray<T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType>(
    <T as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
)
where
    T: Copy;
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for FooArray<T, Buffer>
where
    T: Copy,
    <T as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType> narrow::Length
for FooArray<T, Buffer>
where
    T: Copy,
    <T as narrow::array::ArrayType>::Array<
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
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Extend<Foo<T>> for FooArray<T, Buffer>
where
    T: Copy,
    <T as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<T>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Foo<T>>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|Foo(_0)| {
                self.0.extend(::std::iter::once(_0));
            });
    }
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Foo<T>> for FooArray<T, Buffer>
where
    T: Copy,
    <T as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<T>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Foo<T>>>(iter: _I) -> Self {
        let (_0, ()) = iter.into_iter().map(|Foo(_0)| (_0, ())).unzip();
        Self(_0)
    }
}
struct Bar<'a, T>(&'a Foo<T>);
impl<'a, T: narrow::array::ArrayType> narrow::array::ArrayType for Bar<'a, T> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar<'a, T>, false, Buffer>;
}
impl<'a, T: narrow::array::ArrayType> narrow::array::ArrayType<Bar<'a, T>>
for ::std::option::Option<Bar<'a, T>> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar<'a, T>, true, Buffer>;
}
impl<'a, T: narrow::array::ArrayType> narrow::array::StructArrayType for Bar<'a, T> {
    type Array<Buffer: narrow::buffer::BufferType> = BarArray<'a, T, Buffer>;
}
impl<
    'a,
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> narrow::arrow::StructArrayTypeFields for BarArray<'a, T, Buffer>
where
    <&'a Foo<
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
                <<&'a Foo<
                    T,
                > as ::narrow::array::ArrayType>::Array<
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
> ::std::convert::From<BarArray<'a, T, Buffer>>
for ::std::vec::Vec<::std::sync::Arc<dyn ::arrow_array::Array>>
where
    <&'a Foo<
        T,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<&'a Foo<
            T,
        > as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
{
    fn from(value: BarArray<'a, T, Buffer>) -> Self {
        <[_]>::into_vec(
            #[rustc_box]
            ::alloc::boxed::Box::new([
                ::std::sync::Arc::<
                    <<&'a Foo<
                        T,
                    > as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.0.into()),
            ]),
        )
    }
}
struct BarArray<'a, T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType>(
    <&'a Foo<
        T,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
);
impl<
    'a,
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for BarArray<'a, T, Buffer>
where
    <&'a Foo<
        T,
    > as narrow::array::ArrayType>::Array<
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
for BarArray<'a, T, Buffer>
where
    <&'a Foo<
        T,
    > as narrow::array::ArrayType>::Array<
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
> ::std::iter::Extend<Bar<'a, T>> for BarArray<'a, T, Buffer>
where
    <&'a Foo<
        T,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<&'a Foo<T>>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Bar<'a, T>>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|Bar(_0)| {
                self.0.extend(::std::iter::once(_0));
            });
    }
}
impl<
    'a,
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Bar<'a, T>> for BarArray<'a, T, Buffer>
where
    <&'a Foo<
        T,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<&'a Foo<T>>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Bar<'a, T>>>(iter: _I) -> Self {
        let (_0, ()) = iter.into_iter().map(|Bar(_0)| (_0, ())).unzip();
        Self(_0)
    }
}
struct FooBar<'a>(Bar<'a, u32>);
impl<'a> narrow::array::ArrayType for FooBar<'a> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<FooBar<'a>, false, Buffer>;
}
impl<'a> narrow::array::ArrayType<FooBar<'a>> for ::std::option::Option<FooBar<'a>> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<FooBar<'a>, true, Buffer>;
}
impl<'a> narrow::array::StructArrayType for FooBar<'a> {
    type Array<Buffer: narrow::buffer::BufferType> = FooBarArray<'a, Buffer>;
}
impl<'a, Buffer: narrow::buffer::BufferType> narrow::arrow::StructArrayTypeFields
for FooBarArray<'a, Buffer>
where
    <Bar<
        'a,
        u32,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::arrow::ArrowArray,
{
    fn fields() -> ::arrow_schema::Fields {
        ::arrow_schema::Fields::from([
            ::std::sync::Arc::new(
                <<Bar<
                    'a,
                    u32,
                > as ::narrow::array::ArrayType>::Array<
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
    Buffer: narrow::buffer::BufferType,
> ::std::convert::From<FooBarArray<'a, Buffer>>
for ::std::vec::Vec<::std::sync::Arc<dyn ::arrow_array::Array>>
where
    <Bar<
        'a,
        u32,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::convert::Into<
        <<Bar<
            'a,
            u32,
        > as narrow::array::ArrayType>::Array<
            Buffer,
            narrow::offset::NA,
            narrow::array::union::NA,
        > as narrow::arrow::ArrowArray>::Array,
    >,
{
    fn from(value: FooBarArray<'a, Buffer>) -> Self {
        <[_]>::into_vec(
            #[rustc_box]
            ::alloc::boxed::Box::new([
                ::std::sync::Arc::<
                    <<Bar<
                        'a,
                        u32,
                    > as narrow::array::ArrayType>::Array<
                        Buffer,
                        narrow::offset::NA,
                        narrow::array::union::NA,
                    > as narrow::arrow::ArrowArray>::Array,
                >::new(value.0.into()),
            ]),
        )
    }
}
struct FooBarArray<'a, Buffer: narrow::buffer::BufferType>(
    <Bar<
        'a,
        u32,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >,
);
impl<'a, Buffer: narrow::buffer::BufferType> ::std::default::Default
for FooBarArray<'a, Buffer>
where
    <Bar<
        'a,
        u32,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<'a, Buffer: narrow::buffer::BufferType> narrow::Length for FooBarArray<'a, Buffer>
where
    <Bar<
        'a,
        u32,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<'a, Buffer: narrow::buffer::BufferType> ::std::iter::Extend<FooBar<'a>>
for FooBarArray<'a, Buffer>
where
    <Bar<
        'a,
        u32,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<Bar<'a, u32>>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = FooBar<'a>>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|FooBar(_0)| {
                self.0.extend(::std::iter::once(_0));
            });
    }
}
impl<'a, Buffer: narrow::buffer::BufferType> ::std::iter::FromIterator<FooBar<'a>>
for FooBarArray<'a, Buffer>
where
    <Bar<
        'a,
        u32,
    > as narrow::array::ArrayType>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<Bar<'a, u32>>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = FooBar<'a>>>(iter: _I) -> Self {
        let (_0, ()) = iter.into_iter().map(|FooBar(_0)| (_0, ())).unzip();
        Self(_0)
    }
}
