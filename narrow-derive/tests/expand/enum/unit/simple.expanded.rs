enum FooBar {
    Foo,
    FoO,
    FOO,
    Bar,
}
impl ::std::convert::From<&FooBar> for ::std::primitive::i8 {
    fn from(value: &FooBar) -> i8 {
        match *value {
            FooBar::Foo => 0,
            FooBar::FoO => 1,
            FooBar::FOO => 2,
            FooBar::Bar => 3,
        }
    }
}
impl narrow::array::union::EnumVariant<0> for FooBar {
    type Data = ();
    fn from_data(value: Self::Data) -> Self {
        Self::Foo
    }
}
impl narrow::array::union::EnumVariant<1> for FooBar {
    type Data = ();
    fn from_data(value: Self::Data) -> Self {
        Self::FoO
    }
}
impl narrow::array::union::EnumVariant<2> for FooBar {
    type Data = ();
    fn from_data(value: Self::Data) -> Self {
        Self::FOO
    }
}
impl narrow::array::union::EnumVariant<3> for FooBar {
    type Data = ();
    fn from_data(value: Self::Data) -> Self {
        Self::Bar
    }
}
struct FooBarArray<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
>(
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType>::Array<Buffer, OffsetItem, UnionLayout>,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType>::Array<Buffer, OffsetItem, UnionLayout>,
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType>::Array<Buffer, OffsetItem, UnionLayout>,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType>::Array<Buffer, OffsetItem, UnionLayout>,
);
impl<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
> ::std::default::Default for FooBarArray<Buffer, OffsetItem, UnionLayout>
where
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::std::default::Default,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::std::default::Default,
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::std::default::Default,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
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
impl<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<FooBar> for FooBarArray<Buffer, OffsetItem, DenseLayout>
where
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<()>,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<()>,
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<()>,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<()>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = FooBar>,
    {
        iter.into_iter()
            .for_each(|variant| {
                match variant {
                    FooBar::Foo => {
                        self.0.extend(::std::iter::once(()));
                    }
                    FooBar::FoO => {
                        self.1.extend(::std::iter::once(()));
                    }
                    FooBar::FOO => {
                        self.2.extend(::std::iter::once(()));
                    }
                    FooBar::Bar => {
                        self.3.extend(::std::iter::once(()));
                    }
                }
            });
    }
}
impl<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<FooBar> for FooBarArray<Buffer, OffsetItem, SparseLayout>
where
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<()>,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<()>,
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<()>,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<()>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = FooBar>,
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
                        self.2
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                        self.3
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                    }
                    FooBar::FoO => {
                        self.1.extend(::std::iter::once(()));
                        self.0
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                        self.2
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                        self.3
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                    }
                    FooBar::FOO => {
                        self.2.extend(::std::iter::once(()));
                        self.0
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                        self.1
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                        self.3
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                    }
                    FooBar::Bar => {
                        self.3.extend(::std::iter::once(()));
                        self.0
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                        self.1
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                        self.2
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                    }
                }
            });
    }
}
impl narrow::array::UnionArrayType<4> for FooBar {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = FooBarArray<Buffer, OffsetItem, UnionLayout>;
}
impl narrow::array::ArrayType for FooBar {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::UnionArray<
        Self,
        { <Self as narrow::array::UnionArrayType<4>>::VARIANTS },
        UnionLayout,
        Buffer,
    >;
}
