enum FooBar<const X: bool> {
    Foo,
    Bar,
}
impl<const X: bool> ::std::convert::From<&FooBar<X>> for ::std::primitive::i8 {
    fn from(value: &FooBar<X>) -> i8 {
        match *value {
            FooBar::Foo => 0,
            FooBar::Bar => 1,
        }
    }
}
impl<const X: bool> narrow::array::union::EnumVariant<0> for FooBar<X> {
    type Data = ();
    fn from_data(value: Self::Data) -> Self {
        Self::Foo
    }
}
impl<const X: bool> narrow::array::union::EnumVariant<1> for FooBar<X> {
    type Data = ();
    fn from_data(value: Self::Data) -> Self {
        Self::Bar
    }
}
struct FooBarArray<
    const X: bool,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
>(
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType>::Array<Buffer, OffsetItem, UnionLayout>,
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType>::Array<Buffer, OffsetItem, UnionLayout>,
);
impl<
    const X: bool,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
> ::std::default::Default for FooBarArray<X, Buffer, OffsetItem, UnionLayout>
where
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::std::default::Default,
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default(), ::std::default::Default::default())
    }
}
impl<
    const X: bool,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<FooBar<X>> for FooBarArray<X, Buffer, OffsetItem, DenseLayout>
where
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<()>,
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<()>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = FooBar<X>>,
    {
        iter.into_iter()
            .for_each(|variant| {
                match variant {
                    FooBar::Foo => {
                        self.0.extend(::std::iter::once(()));
                    }
                    FooBar::Bar => {
                        self.1.extend(::std::iter::once(()));
                    }
                }
            });
    }
}
impl<
    const X: bool,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<FooBar<X>> for FooBarArray<X, Buffer, OffsetItem, SparseLayout>
where
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<()>,
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<()>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = FooBar<X>>,
    {
        iter.into_iter()
            .for_each(|variant| {
                match variant {
                    FooBar::Foo => {
                        self.0.extend(::std::iter::once(()));
                        self.1
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                    }
                    FooBar::Bar => {
                        self.1.extend(::std::iter::once(()));
                        self.0
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                    }
                }
            });
    }
}
impl<const X: bool> narrow::array::UnionArrayType<2> for FooBar<X> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = FooBarArray<X, Buffer, OffsetItem, UnionLayout>;
}
impl<const X: bool> narrow::array::ArrayType for FooBar<X> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::UnionArray<
        Self,
        { <Self as narrow::array::UnionArrayType<2>>::VARIANTS },
        UnionLayout,
        Buffer,
    >;
}
