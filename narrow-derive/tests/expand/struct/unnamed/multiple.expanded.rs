struct Bar(u8, u16, u32, u64);
impl narrow::array::ArrayType<Bar> for Bar {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar, narrow::NonNullable, Buffer>;
}
impl narrow::array::ArrayType<Bar> for ::std::option::Option<Bar> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar, narrow::Nullable, Buffer>;
}
impl narrow::array::StructArrayType for Bar {
    type Array<Buffer: narrow::buffer::BufferType> = BarArray<Buffer>;
}
struct BarArray<Buffer: narrow::buffer::BufferType>(
    <u8 as narrow::array::ArrayType<
        u8,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
    <u16 as narrow::array::ArrayType<
        u16,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
    <u64 as narrow::array::ArrayType<
        u64,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
);
impl<Buffer: narrow::buffer::BufferType> ::std::clone::Clone for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType<
        u8,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
    <u16 as narrow::array::ArrayType<
        u16,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
    <u64 as narrow::array::ArrayType<
        u64,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), self.2.clone(), self.3.clone())
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::default::Default for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType<
        u8,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <u16 as narrow::array::ArrayType<
        u16,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <u64 as narrow::array::ArrayType<
        u64,
    >>::Array<
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
    <u8 as narrow::array::ArrayType<
        u8,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
    <u16 as narrow::array::ArrayType<
        u16,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
    <u64 as narrow::array::ArrayType<
        u64,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Extend<Bar> for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType<
        u8,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<u8>,
    <u16 as narrow::array::ArrayType<
        u16,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<u16>,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<u32>,
    <u64 as narrow::array::ArrayType<
        u64,
    >>::Array<
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
    <u8 as narrow::array::ArrayType<
        u8,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<u8>,
    <u16 as narrow::array::ArrayType<
        u16,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<u16>,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<u32>,
    <u64 as narrow::array::ArrayType<
        u64,
    >>::Array<
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
struct BarArrayIter<Buffer: narrow::buffer::BufferType>(
    <<u8 as narrow::array::ArrayType<
        u8,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
    <<u16 as narrow::array::ArrayType<
        u16,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
    <<u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
    <<u64 as narrow::array::ArrayType<
        u64,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
)
where
    <u8 as narrow::array::ArrayType<
        u8,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u8>,
    <u16 as narrow::array::ArrayType<
        u16,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u16>,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u32>,
    <u64 as narrow::array::ArrayType<
        u64,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u64>;
impl<Buffer: narrow::buffer::BufferType> ::std::iter::Iterator for BarArrayIter<Buffer>
where
    <u8 as narrow::array::ArrayType<
        u8,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u8>,
    <u16 as narrow::array::ArrayType<
        u16,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u16>,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u32>,
    <u64 as narrow::array::ArrayType<
        u64,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u64>,
{
    type Item = Bar;
    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|first| {
                Bar(
                    first,
                    self.1.next().unwrap(),
                    self.2.next().unwrap(),
                    self.3.next().unwrap(),
                )
            })
    }
}
impl<Buffer: narrow::buffer::BufferType> ::std::iter::IntoIterator for BarArray<Buffer>
where
    <u8 as narrow::array::ArrayType<
        u8,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u8>,
    <u16 as narrow::array::ArrayType<
        u16,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u16>,
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u32>,
    <u64 as narrow::array::ArrayType<
        u64,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u64>,
{
    type Item = Bar;
    type IntoIter = BarArrayIter<Buffer>;
    fn into_iter(self) -> Self::IntoIter {
        BarArrayIter(
            self.0.into_iter(),
            self.1.into_iter(),
            self.2.into_iter(),
            self.3.into_iter(),
        )
    }
}
