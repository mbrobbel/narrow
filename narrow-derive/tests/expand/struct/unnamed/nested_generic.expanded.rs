struct Foo<T>(
    T,
)
where
    T: Copy;
impl<T: narrow::array::ArrayType<T>> narrow::array::ArrayType<Foo<T>> for Foo<T>
where
    T: Copy,
{
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<T>, narrow::NonNullable, Buffer>;
}
impl<T: narrow::array::ArrayType<T>> narrow::array::ArrayType<Foo<T>>
for ::std::option::Option<Foo<T>>
where
    T: Copy,
{
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<T>, narrow::Nullable, Buffer>;
}
impl<T: narrow::array::ArrayType<T>> narrow::array::StructArrayType for Foo<T>
where
    T: Copy,
{
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<T, Buffer>;
}
struct FooArray<T: narrow::array::ArrayType<T>, Buffer: narrow::buffer::BufferType>(
    <T as narrow::array::ArrayType<
        T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
)
where
    T: Copy;
impl<
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::clone::Clone for FooArray<T, Buffer>
where
    T: Copy,
    <T as narrow::array::ArrayType<
        T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for FooArray<T, Buffer>
where
    T: Copy,
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<T: narrow::array::ArrayType<T>, Buffer: narrow::buffer::BufferType> narrow::Length
for FooArray<T, Buffer>
where
    T: Copy,
    <T as narrow::array::ArrayType<
        T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Extend<Foo<T>> for FooArray<T, Buffer>
where
    T: Copy,
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
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
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Foo<T>> for FooArray<T, Buffer>
where
    T: Copy,
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
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
struct FooArrayIter<T: narrow::array::ArrayType<T>, Buffer: narrow::buffer::BufferType>(
    <<T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
)
where
    T: Copy,
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = T>;
impl<
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Iterator for FooArrayIter<T, Buffer>
where
    T: Copy,
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = T>,
{
    type Item = Foo<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|first| { Foo(first) })
    }
}
impl<
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::IntoIterator for FooArray<T, Buffer>
where
    T: Copy,
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = T>,
{
    type Item = Foo<T>;
    type IntoIter = FooArrayIter<T, Buffer>;
    fn into_iter(self) -> Self::IntoIter {
        FooArrayIter(self.0.into_iter())
    }
}
struct Bar<'a, T>(&'a Foo<T>);
impl<'a, T: narrow::array::ArrayType<T>> narrow::array::ArrayType<Bar<'a, T>>
for Bar<'a, T> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar<'a, T>, narrow::NonNullable, Buffer>;
}
impl<'a, T: narrow::array::ArrayType<T>> narrow::array::ArrayType<Bar<'a, T>>
for ::std::option::Option<Bar<'a, T>> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar<'a, T>, narrow::Nullable, Buffer>;
}
impl<'a, T: narrow::array::ArrayType<T>> narrow::array::StructArrayType for Bar<'a, T> {
    type Array<Buffer: narrow::buffer::BufferType> = BarArray<'a, T, Buffer>;
}
struct BarArray<'a, T: narrow::array::ArrayType<T>, Buffer: narrow::buffer::BufferType>(
    <&'a Foo<
        T,
    > as narrow::array::ArrayType<
        &'a Foo<T>,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
);
impl<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::clone::Clone for BarArray<'a, T, Buffer>
where
    <&'a Foo<
        T,
    > as narrow::array::ArrayType<
        &'a Foo<T>,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for BarArray<'a, T, Buffer>
where
    <&'a Foo<
        T,
    > as narrow::array::ArrayType<
        &'a Foo<T>,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default())
    }
}
impl<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> narrow::Length for BarArray<'a, T, Buffer>
where
    <&'a Foo<
        T,
    > as narrow::array::ArrayType<
        &'a Foo<T>,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Extend<Bar<'a, T>> for BarArray<'a, T, Buffer>
where
    <&'a Foo<
        T,
    > as narrow::array::ArrayType<
        &'a Foo<T>,
    >>::Array<
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
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Bar<'a, T>> for BarArray<'a, T, Buffer>
where
    <&'a Foo<
        T,
    > as narrow::array::ArrayType<
        &'a Foo<T>,
    >>::Array<
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
struct BarArrayIter<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
>(
    <<&'a Foo<
        T,
    > as narrow::array::ArrayType<
        &'a Foo<T>,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
)
where
    <&'a Foo<
        T,
    > as narrow::array::ArrayType<
        &'a Foo<T>,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = &'a Foo<T>>;
impl<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Iterator for BarArrayIter<'a, T, Buffer>
where
    <&'a Foo<
        T,
    > as narrow::array::ArrayType<
        &'a Foo<T>,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = &'a Foo<T>>,
{
    type Item = Bar<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|first| { Bar(first) })
    }
}
impl<
    'a,
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::IntoIterator for BarArray<'a, T, Buffer>
where
    <&'a Foo<
        T,
    > as narrow::array::ArrayType<
        &'a Foo<T>,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = &'a Foo<T>>,
{
    type Item = Bar<'a, T>;
    type IntoIter = BarArrayIter<'a, T, Buffer>;
    fn into_iter(self) -> Self::IntoIter {
        BarArrayIter(self.0.into_iter())
    }
}
struct FooBar<'a>(Bar<'a, u32>);
impl<'a> narrow::array::ArrayType<FooBar<'a>> for FooBar<'a> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<FooBar<'a>, narrow::NonNullable, Buffer>;
}
impl<'a> narrow::array::ArrayType<FooBar<'a>> for ::std::option::Option<FooBar<'a>> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<FooBar<'a>, narrow::Nullable, Buffer>;
}
impl<'a> narrow::array::StructArrayType for FooBar<'a> {
    type Array<Buffer: narrow::buffer::BufferType> = FooBarArray<'a, Buffer>;
}
struct FooBarArray<'a, Buffer: narrow::buffer::BufferType>(
    <Bar<
        'a,
        u32,
    > as narrow::array::ArrayType<
        Bar<'a, u32>,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
);
impl<'a, Buffer: narrow::buffer::BufferType> ::std::clone::Clone
for FooBarArray<'a, Buffer>
where
    <Bar<
        'a,
        u32,
    > as narrow::array::ArrayType<
        Bar<'a, u32>,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<'a, Buffer: narrow::buffer::BufferType> ::std::default::Default
for FooBarArray<'a, Buffer>
where
    <Bar<
        'a,
        u32,
    > as narrow::array::ArrayType<
        Bar<'a, u32>,
    >>::Array<
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
    > as narrow::array::ArrayType<
        Bar<'a, u32>,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
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
    > as narrow::array::ArrayType<
        Bar<'a, u32>,
    >>::Array<
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
    > as narrow::array::ArrayType<
        Bar<'a, u32>,
    >>::Array<
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
struct FooBarArrayIter<'a, Buffer: narrow::buffer::BufferType>(
    <<Bar<
        'a,
        u32,
    > as narrow::array::ArrayType<
        Bar<'a, u32>,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
)
where
    <Bar<
        'a,
        u32,
    > as narrow::array::ArrayType<
        Bar<'a, u32>,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = Bar<'a, u32>>;
impl<'a, Buffer: narrow::buffer::BufferType> ::std::iter::Iterator
for FooBarArrayIter<'a, Buffer>
where
    <Bar<
        'a,
        u32,
    > as narrow::array::ArrayType<
        Bar<'a, u32>,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = Bar<'a, u32>>,
{
    type Item = FooBar<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|first| { FooBar(first) })
    }
}
impl<'a, Buffer: narrow::buffer::BufferType> ::std::iter::IntoIterator
for FooBarArray<'a, Buffer>
where
    <Bar<
        'a,
        u32,
    > as narrow::array::ArrayType<
        Bar<'a, u32>,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = Bar<'a, u32>>,
{
    type Item = FooBar<'a>;
    type IntoIter = FooBarArrayIter<'a, Buffer>;
    fn into_iter(self) -> Self::IntoIter {
        FooBarArrayIter(self.0.into_iter())
    }
}
