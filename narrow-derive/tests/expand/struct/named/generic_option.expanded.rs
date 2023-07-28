struct Bar<T> {
    a: u32,
    b: Option<bool>,
    c: Option<T>,
}
impl<T: narrow::array::ArrayType> narrow::array::ArrayType for Bar<T> {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Bar<T>,
        false,
        Buffer,
    >;
}
impl<T: narrow::array::ArrayType> narrow::array::ArrayType<Bar<T>>
for ::std::option::Option<Bar<T>> {
    type Array<Buffer: narrow::buffer::BufferType> = narrow::array::StructArray<
        Bar<T>,
        true,
        Buffer,
    >;
}
impl<T: narrow::array::ArrayType> narrow::array::StructArrayType for Bar<T>
where
    u32: narrow::array::ArrayType,
    Option<bool>: narrow::array::ArrayType,
    Option<T>: narrow::array::ArrayType,
{
    type Array<Buffer: narrow::buffer::BufferType> = BarArray<T, Buffer>;
}
struct BarArray<T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType>
where
    u32: narrow::array::ArrayType,
    Option<bool>: narrow::array::ArrayType,
    Option<T>: narrow::array::ArrayType,
{
    a: <u32 as narrow::array::ArrayType>::Array<Buffer>,
    b: <Option<bool> as narrow::array::ArrayType>::Array<Buffer>,
    c: <Option<T> as narrow::array::ArrayType>::Array<Buffer>,
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Bar<T>> for BarArray<T, Buffer>
where
    u32: narrow::array::ArrayType,
    Option<bool>: narrow::array::ArrayType,
    Option<T>: narrow::array::ArrayType,
    <u32 as narrow::array::ArrayType>::Array<
        Buffer,
    >: ::std::default::Default + ::std::iter::Extend<u32>,
    <Option<
        bool,
    > as narrow::array::ArrayType>::Array<
        Buffer,
    >: ::std::default::Default + ::std::iter::Extend<Option<bool>>,
    <Option<
        T,
    > as narrow::array::ArrayType>::Array<
        Buffer,
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
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for BarArray<T, Buffer>
where
    u32: narrow::array::ArrayType,
    Option<bool>: narrow::array::ArrayType,
    Option<T>: narrow::array::ArrayType,
    <u32 as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
    <Option<bool> as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
    <Option<T> as narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default,
{
    fn default() -> Self {
        Self {
            a: ::std::default::Default::default(),
            b: ::std::default::Default::default(),
            c: ::std::default::Default::default(),
        }
    }
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Extend<Bar<T>> for BarArray<T, Buffer>
where
    u32: narrow::array::ArrayType,
    Option<bool>: narrow::array::ArrayType,
    Option<T>: narrow::array::ArrayType,
    <u32 as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<u32>,
    <Option<
        bool,
    > as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<Option<bool>>,
    <Option<
        T,
    > as narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<Option<T>>,
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
impl<T: narrow::array::ArrayType, Buffer: narrow::buffer::BufferType> narrow::Length
for BarArray<T, Buffer>
where
    u32: narrow::array::ArrayType,
    <u32 as narrow::array::ArrayType>::Array<Buffer>: narrow::Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.a.len()
    }
}
