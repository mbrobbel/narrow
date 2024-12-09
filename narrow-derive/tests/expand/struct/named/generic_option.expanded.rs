struct Bar<T> {
    a: u32,
    b: Option<bool>,
    c: Option<T>,
}
impl<T: narrow::array::ArrayType<T>> narrow::array::ArrayType<Bar<T>> for Bar<T> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar<T>, narrow::NonNullable, Buffer>;
}
impl<T: narrow::array::ArrayType<T>> narrow::array::ArrayType<Bar<T>>
for ::std::option::Option<Bar<T>> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Bar<T>, narrow::Nullable, Buffer>;
}
impl<T: narrow::array::ArrayType<T>> narrow::array::StructArrayType for Bar<T> {
    type Array<Buffer: narrow::buffer::BufferType> = BarArray<T, Buffer>;
}
struct BarArray<T: narrow::array::ArrayType<T>, Buffer: narrow::buffer::BufferType> {
    a: <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
    b: <Option<
        bool,
    > as narrow::array::ArrayType<
        bool,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
    c: <Option<
        T,
    > as narrow::array::ArrayType<
        T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
}
impl<
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::clone::Clone for BarArray<T, Buffer>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
    <Option<
        bool,
    > as narrow::array::ArrayType<
        bool,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
    <Option<
        T,
    > as narrow::array::ArrayType<
        T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: ::std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self {
            a: self.a.clone(),
            b: self.b.clone(),
            c: self.c.clone(),
        }
    }
}
impl<
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for BarArray<T, Buffer>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <Option<
        bool,
    > as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default,
    <Option<
        T,
    > as narrow::array::ArrayType<
        T,
    >>::Array<
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
impl<T: narrow::array::ArrayType<T>, Buffer: narrow::buffer::BufferType> narrow::Length
for BarArray<T, Buffer>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
    <Option<
        bool,
    > as narrow::array::ArrayType<
        bool,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
    <Option<
        T,
    > as narrow::array::ArrayType<
        T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
{
    fn len(&self) -> usize {
        self.a.len()
    }
}
impl<
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Extend<Bar<T>> for BarArray<T, Buffer>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<u32>,
    <Option<
        bool,
    > as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<Option<bool>>,
    <Option<
        T,
    > as narrow::array::ArrayType<
        T,
    >>::Array<
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
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Bar<T>> for BarArray<T, Buffer>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<u32>,
    <Option<
        bool,
    > as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<Option<bool>>,
    <Option<
        T,
    > as narrow::array::ArrayType<
        T,
    >>::Array<
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
struct BarArrayIter<T: narrow::array::ArrayType<T>, Buffer: narrow::buffer::BufferType>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u32>,
    <Option<
        bool,
    > as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = Option<bool>>,
    <Option<
        T,
    > as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = Option<T>>,
{
    a: <<u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
    b: <<Option<
        bool,
    > as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
    c: <<Option<
        T,
    > as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    > as ::std::iter::IntoIterator>::IntoIter,
}
impl<
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Iterator for BarArrayIter<T, Buffer>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u32>,
    <Option<
        bool,
    > as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = Option<bool>>,
    <Option<
        T,
    > as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = Option<T>>,
{
    type Item = Bar<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.a
            .next()
            .map(|a| {
                Bar {
                    a,
                    b: self.b.next().unwrap(),
                    c: self.c.next().unwrap(),
                }
            })
    }
}
impl<
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::IntoIterator for BarArray<T, Buffer>
where
    <u32 as narrow::array::ArrayType<
        u32,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = u32>,
    <Option<
        bool,
    > as narrow::array::ArrayType<
        bool,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = Option<bool>>,
    <Option<
        T,
    > as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::IntoIterator<Item = Option<T>>,
{
    type Item = Bar<T>;
    type IntoIter = BarArrayIter<T, Buffer>;
    fn into_iter(self) -> Self::IntoIter {
        BarArrayIter {
            a: self.a.into_iter(),
            b: self.b.into_iter(),
            c: self.c.into_iter(),
        }
    }
}
