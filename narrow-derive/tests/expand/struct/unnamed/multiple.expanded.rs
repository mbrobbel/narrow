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
impl<Buffer: narrow::buffer::BufferType> narrow::Length for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType>::Array<Buffer>: narrow::Length,
    <u16 as narrow::array::ArrayType>::Array<Buffer>: narrow::Length,
    <u32 as narrow::array::ArrayType>::Array<Buffer>: narrow::Length,
    <u64 as narrow::array::ArrayType>::Array<Buffer>: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
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
