struct Foo(u32);
impl narrow::array::ArrayType for Foo {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Foo,
        false,
        Buffer,
    >;
}
impl narrow::array::ArrayType<Foo> for ::std::option::Option<Foo> {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Foo,
        true,
        Buffer,
    >;
}
impl narrow::array::StructArrayType for Foo {
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<Buffer>;
}
struct FooArray<Buffer: narrow::buffer::BufferType>(
    <u32 as narrow::array::ArrayType>::Array<Buffer>,
);
impl<Buffer: narrow::buffer::BufferType> ::std::iter::FromIterator<Foo>
for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
    >: ::std::default::Default + ::std::iter::Extend<u32>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = Foo>>(iter: _I) -> Self {
        let (_0, ()) = iter.into_iter().map(|Foo(_0)| (_0, ())).unzip();
        Self(_0)
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::default::Default for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Foo> for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<u32>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = Foo>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|Foo(_0)| {
                self.0.extend(::std::iter::once(_0));
            });
    }
}
impl<Buffer: narrow::buffer::BufferType> narrow::Length for FooArray<Buffer>
where
    <u32 as narrow::array::ArrayType>::Array<Buffer>: narrow::Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
struct Bar(u8, u16, u32, u64);
impl narrow::array::ArrayType for Bar {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Bar,
        false,
        Buffer,
    >;
}
impl narrow::array::ArrayType<Bar> for ::std::option::Option<Bar> {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Bar,
        true,
        Buffer,
    >;
}
impl narrow::array::StructArrayType for Bar {
    type Array<Buffer: narrow::buffer::BufferType> = BarArray<Buffer>;
}
struct BarArray<Buffer: narrow::buffer::BufferType>(
    <u8 as narrow::array::ArrayType>::Array<Buffer>,
    <u16 as narrow::array::ArrayType>::Array<Buffer>,
    <u32 as narrow::array::ArrayType>::Array<Buffer>,
    <u64 as narrow::array::ArrayType>::Array<Buffer>,
);
impl<Buffer: narrow::buffer::BufferType> ::std::iter::FromIterator<Bar>
for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType>::Array<
        Buffer,
    >: ::std::default::Default + ::std::iter::Extend<u8>,
    <u16 as narrow::array::ArrayType>::Array<
        Buffer,
    >: ::std::default::Default + ::std::iter::Extend<u16>,
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
    >: ::std::default::Default + ::std::iter::Extend<u32>,
    <u64 as narrow::array::ArrayType>::Array<
        Buffer,
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
impl<Buffer: narrow::buffer::BufferType> ::std::default::Default for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
    <u16 as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
    <u32 as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
    <u64 as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
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
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Bar> for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<u8>,
    <u16 as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<u16>,
    <u32 as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<u32>,
    <u64 as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<u64>,
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
impl<Buffer: narrow::buffer::BufferType> narrow::Length for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType>::Array<Buffer>: narrow::Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
struct FooBar<T>(T);
impl<T: narrow::array::ArrayType> narrow::array::ArrayType for FooBar<T> {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        FooBar<T>,
        false,
        Buffer,
    >;
}
impl<T: narrow::array::ArrayType> narrow::array::ArrayType<FooBar<T>>
for ::std::option::Option<FooBar<T>> {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        FooBar<T>,
        true,
        Buffer,
    >;
}
impl<T: narrow::array::ArrayType> narrow::array::StructArrayType for FooBar<T> {
    type Array<Buffer: narrow::buffer::BufferType> = FooBarArray<T, Buffer>;
}
struct FooBarArray<T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType>(
    <T as narrow::array::ArrayType>::Array<Buffer>,
);
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<FooBar<T>> for FooBarArray<T, Buffer>
where
    <T as narrow::array::ArrayType>::Array<
        Buffer,
    >: ::std::default::Default + ::std::iter::Extend<T>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = FooBar<T>>>(iter: _I) -> Self {
        let (_0, ()) = iter.into_iter().map(|FooBar(_0)| (_0, ())).unzip();
        Self(_0)
    }
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for FooBarArray<T, Buffer>
where
    <T as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Extend<FooBar<T>> for FooBarArray<T, Buffer>
where
    <T as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<T>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = FooBar<T>>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|FooBar(_0)| {
                self.0.extend(::std::iter::once(_0));
            });
    }
}
impl<T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType> narrow::Length
for FooBarArray<T, Buffer>
where
    <T as narrow::array::ArrayType>::Array<Buffer>: narrow::Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
