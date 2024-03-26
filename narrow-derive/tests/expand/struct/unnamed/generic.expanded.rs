struct Foo<'a, T: Add<Self>>(
    &'a T,
)
where
    Self: Sized,
    <T as Add<Self>>::Output: Debug;
impl<'a, T: Add<Self> + narrow::array::ArrayType<T>> narrow::array::ArrayType<Foo<'a, T>>
for Foo<'a, T>
where
    Self: Sized,
    <T as Add<Self>>::Output: Debug,
{
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<'a, T>, false, Buffer>;
}
impl<'a, T: Add<Self> + narrow::array::ArrayType<T>> narrow::array::ArrayType<Foo<'a, T>>
for ::std::option::Option<Foo<'a, T>>
where
    Self: Sized,
    <T as Add<Self>>::Output: Debug,
{
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<Foo<'a, T>, true, Buffer>;
}
impl<'a, T: Add<Self> + narrow::array::ArrayType<T>> narrow::array::StructArrayType
for Foo<'a, T>
where
    Self: Sized,
    <T as Add<Self>>::Output: Debug,
{
    type Array<Buffer: narrow::buffer::BufferType> = FooArray<'a, T, Buffer>;
}
struct FooArray<
    'a,
    T: Add<Foo<'a, T>> + narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
>(
    <&'a T as narrow::array::ArrayType<
        &'a T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
)
where
    Foo<'a, T>: Sized,
    <T as Add<Foo<'a, T>>>::Output: Debug;
impl<
    'a,
    T: Add<Foo<'a, T>> + narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for FooArray<'a, T, Buffer>
where
    Foo<'a, T>: Sized,
    <T as Add<Foo<'a, T>>>::Output: Debug,
    <&'a T as narrow::array::ArrayType<
        &'a T,
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
    T: Add<Foo<'a, T>> + narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> narrow::Length for FooArray<'a, T, Buffer>
where
    Foo<'a, T>: Sized,
    <T as Add<Foo<'a, T>>>::Output: Debug,
    <&'a T as narrow::array::ArrayType<
        &'a T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>: narrow::Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<
    'a,
    T: Add<Self> + narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::Extend<Foo<'a, T>> for FooArray<'a, T, Buffer>
where
    Self: Sized,
    <T as Add<Self>>::Output: Debug,
    <&'a T as narrow::array::ArrayType<
        &'a T,
    >>::Array<
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
    T: Add<Self> + narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<Foo<'a, T>> for FooArray<'a, T, Buffer>
where
    Self: Sized,
    <T as Add<Self>>::Output: Debug,
    <&'a T as narrow::array::ArrayType<
        &'a T,
    >>::Array<
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
struct FooBar<T>(T);
impl<T: narrow::array::ArrayType<T>> narrow::array::ArrayType<FooBar<T>> for FooBar<T> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<FooBar<T>, false, Buffer>;
}
impl<T: narrow::array::ArrayType<T>> narrow::array::ArrayType<FooBar<T>>
for ::std::option::Option<FooBar<T>> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::StructArray<FooBar<T>, true, Buffer>;
}
impl<T: narrow::array::ArrayType<T>> narrow::array::StructArrayType for FooBar<T> {
    type Array<Buffer: narrow::buffer::BufferType> = FooBarArray<T, Buffer>;
}
struct FooBarArray<T: narrow::array::ArrayType<T>, Buffer: narrow::buffer::BufferType>(
    <T as narrow::array::ArrayType<
        T,
    >>::Array<Buffer, narrow::offset::NA, narrow::array::union::NA>,
);
impl<
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::default::Default for FooBarArray<T, Buffer>
where
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
for FooBarArray<T, Buffer>
where
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
> ::std::iter::Extend<FooBar<T>> for FooBarArray<T, Buffer>
where
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::iter::Extend<T>,
{
    fn extend<_I: ::std::iter::IntoIterator<Item = FooBar<T>>>(&mut self, iter: _I) {
        iter.into_iter()
            .for_each(|FooBar(_0)| {
                self.0.extend(::std::iter::once(_0));
            });
    }
}
impl<
    T: narrow::array::ArrayType<T>,
    Buffer: narrow::buffer::BufferType,
> ::std::iter::FromIterator<FooBar<T>> for FooBarArray<T, Buffer>
where
    <T as narrow::array::ArrayType<
        T,
    >>::Array<
        Buffer,
        narrow::offset::NA,
        narrow::array::union::NA,
    >: ::std::default::Default + ::std::iter::Extend<T>,
{
    fn from_iter<_I: ::std::iter::IntoIterator<Item = FooBar<T>>>(iter: _I) -> Self {
        let (_0, ()) = iter.into_iter().map(|FooBar(_0)| (_0, ())).unzip();
        Self(_0)
    }
}
