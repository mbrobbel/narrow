enum FooBar {
    Unit,
    Foo { bar: u32 },
    Bar { foo: bool },
    FooBar { foo: String, bar: Option<u8> },
}
impl ::std::convert::From<&FooBar> for ::std::primitive::i8 {
    fn from(value: &FooBar) -> i8 {
        match *value {
            FooBar::Unit => 0,
            FooBar::Foo { .. } => 1,
            FooBar::Bar { .. } => 2,
            FooBar::FooBar { .. } => 3,
        }
    }
}
struct FooBarVariantFoo {
    bar: u32,
}
#[automatically_derived]
impl ::core::default::Default for FooBarVariantFoo {
    #[inline]
    fn default() -> FooBarVariantFoo {
        FooBarVariantFoo {
            bar: ::core::default::Default::default(),
        }
    }
}
struct FooBarVariantBar {
    foo: bool,
}
#[automatically_derived]
impl ::core::default::Default for FooBarVariantBar {
    #[inline]
    fn default() -> FooBarVariantBar {
        FooBarVariantBar {
            foo: ::core::default::Default::default(),
        }
    }
}
struct FooBarVariantFooBar {
    foo: String,
    bar: Option<u8>,
}
#[automatically_derived]
impl ::core::default::Default for FooBarVariantFooBar {
    #[inline]
    fn default() -> FooBarVariantFooBar {
        FooBarVariantFooBar {
            foo: ::core::default::Default::default(),
            bar: ::core::default::Default::default(),
        }
    }
}
impl narrow::array::union::EnumVariant<0> for FooBar {
    type Data = ();
    fn from_data(value: Self::Data) -> Self {
        Self::Unit
    }
}
impl narrow::array::union::EnumVariant<1> for FooBar {
    type Data = FooBarVariantFoo;
    fn from_data(value: Self::Data) -> Self {
        Self::Foo { bar: value.bar }
    }
}
impl narrow::array::union::EnumVariant<2> for FooBar {
    type Data = FooBarVariantBar;
    fn from_data(value: Self::Data) -> Self {
        Self::Bar { foo: value.foo }
    }
}
impl narrow::array::union::EnumVariant<3> for FooBar {
    type Data = FooBarVariantFooBar;
    fn from_data(value: Self::Data) -> Self {
        Self::FooBar {
            foo: value.foo,
            bar: value.bar,
        }
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
    >: ::std::iter::Extend<FooBarVariantFoo>,
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<FooBarVariantBar>,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<FooBarVariantFooBar>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = FooBar>,
    {
        iter.into_iter()
            .for_each(|variant| {
                match variant {
                    FooBar::Unit => {
                        self.0.extend(::std::iter::once(()));
                    }
                    FooBar::Foo { bar } => {
                        self.1.extend(::std::iter::once(FooBarVariantFoo { bar }));
                    }
                    FooBar::Bar { foo } => {
                        self.2.extend(::std::iter::once(FooBarVariantBar { foo }));
                    }
                    FooBar::FooBar { foo, bar } => {
                        self.3
                            .extend(::std::iter::once(FooBarVariantFooBar { foo, bar }));
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
    >: ::std::iter::Extend<FooBarVariantFoo>,
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<FooBarVariantBar>,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<FooBarVariantFooBar>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = FooBar>,
    {
        iter.into_iter()
            .for_each(|variant| {
                match variant {
                    FooBar::Unit => {
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
                    FooBar::Foo { bar } => {
                        self.1.extend(::std::iter::once(FooBarVariantFoo { bar }));
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
                    FooBar::Bar { foo } => {
                        self.2.extend(::std::iter::once(FooBarVariantBar { foo }));
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
                    FooBar::FooBar { foo, bar } => {
                        self.3
                            .extend(::std::iter::once(FooBarVariantFooBar { foo, bar }));
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
