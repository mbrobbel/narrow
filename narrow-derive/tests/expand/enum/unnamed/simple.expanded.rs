enum FooBar {
    Foo(bool),
    Bar(u8, u16),
}
impl ::std::convert::From<&FooBar> for ::std::primitive::i8 {
    fn from(value: &FooBar) -> ::std::primitive::i8 {
        match *value {
            FooBar::Foo(..) => 0,
            FooBar::Bar(..) => 1,
        }
    }
}
struct FooBarVariantFoo(bool);
#[automatically_derived]
impl ::core::default::Default for FooBarVariantFoo {
    #[inline]
    fn default() -> FooBarVariantFoo {
        FooBarVariantFoo(::core::default::Default::default())
    }
}
struct FooBarVariantBar(u8, u16);
#[automatically_derived]
impl ::core::default::Default for FooBarVariantBar {
    #[inline]
    fn default() -> FooBarVariantBar {
        FooBarVariantBar(
            ::core::default::Default::default(),
            ::core::default::Default::default(),
        )
    }
}
impl narrow::array::union::EnumVariant<0> for FooBar {
    type Data = FooBarVariantFoo;
    fn from_data(value: Self::Data) -> Self {
        Self::Foo(value.0)
    }
}
impl narrow::array::union::EnumVariant<1> for FooBar {
    type Data = FooBarVariantBar;
    fn from_data(value: Self::Data) -> Self {
        Self::Bar(value.0, value.1)
    }
}
struct FooBarArray<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
>(
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>,
);
impl<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
> ::std::default::Default for FooBarArray<Buffer, OffsetItem, UnionLayout>
where
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::default::Default,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default(), ::std::default::Default::default())
    }
}
impl<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<FooBar>
for FooBarArray<Buffer, OffsetItem, narrow::array::DenseLayout>
where
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<FooBarVariantFoo>,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<FooBarVariantBar>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = FooBar>,
    {
        iter.into_iter()
            .for_each(|variant| {
                match variant {
                    FooBar::Foo(_0) => {
                        self.0.extend(::std::iter::once(FooBarVariantFoo(_0)));
                    }
                    FooBar::Bar(_0, _1) => {
                        self.1.extend(::std::iter::once(FooBarVariantBar(_0, _1)));
                    }
                }
            });
    }
}
impl<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<FooBar>
for FooBarArray<Buffer, OffsetItem, narrow::array::SparseLayout>
where
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<FooBarVariantFoo>,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<FooBarVariantBar>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = FooBar>,
    {
        iter.into_iter()
            .for_each(|variant| {
                match variant {
                    FooBar::Foo(_0) => {
                        self.0.extend(::std::iter::once(FooBarVariantFoo(_0)));
                        self.1
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                    }
                    FooBar::Bar(_0, _1) => {
                        self.1.extend(::std::iter::once(FooBarVariantBar(_0, _1)));
                        self.0
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                    }
                }
            });
    }
}
struct FooBarArrayIntoIter<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
>(
    <<<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout> as ::core::iter::IntoIterator>::IntoIter,
    <<<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout> as ::core::iter::IntoIterator>::IntoIter,
)
where
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::core::iter::IntoIterator,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::core::iter::IntoIterator;
impl<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> narrow::array::union::TypeIdIterator
for FooBarArrayIntoIter<Buffer, OffsetItem, narrow::array::DenseLayout>
where
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >,
{
    type Enum = FooBar;
    fn next(
        &mut self,
        type_id: ::std::primitive::i8,
    ) -> ::core::option::Option<Self::Enum> {
        match type_id {
            0 => {
                self
                    .0
                    .next()
                    .map(<FooBar as narrow::array::union::EnumVariant<0>>::from_data)
            }
            1 => {
                self
                    .1
                    .next()
                    .map(<FooBar as narrow::array::union::EnumVariant<1>>::from_data)
            }
            _ => {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("type id greater than number of variants"),
                    );
                };
            }
        }
    }
}
impl<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> narrow::array::union::TypeIdIterator
for FooBarArrayIntoIter<Buffer, OffsetItem, narrow::array::SparseLayout>
where
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >,
{
    type Enum = FooBar;
    fn next(
        &mut self,
        type_id: ::std::primitive::i8,
    ) -> ::core::option::Option<Self::Enum> {
        match type_id {
            0 => {
                self.1.next();
                self.0
                    .next()
                    .map(<FooBar as narrow::array::union::EnumVariant<0>>::from_data)
            }
            1 => {
                self.0.next();
                self.1
                    .next()
                    .map(<FooBar as narrow::array::union::EnumVariant<1>>::from_data)
            }
            _ => {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("type id greater than number of variants"),
                    );
                };
            }
        }
    }
}
impl<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
> narrow::array::union::UnionArrayIterators
for FooBarArray<Buffer, OffsetItem, UnionLayout>
where
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >,
    FooBarArrayIntoIter<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: narrow::array::union::TypeIdIterator,
{
    type VariantIterators = FooBarArrayIntoIter<Buffer, OffsetItem, UnionLayout>;
    fn new_variant_iters(self) -> Self::VariantIterators {
        FooBarArrayIntoIter::<
            Buffer,
            OffsetItem,
            UnionLayout,
        >(self.0.into_iter(), self.1.into_iter())
    }
}
impl narrow::array::UnionArrayType<2> for FooBar {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = FooBarArray<Buffer, OffsetItem, UnionLayout>;
}
impl narrow::array::ArrayType<FooBar> for FooBar {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::UnionArray<
        Self,
        { <Self as narrow::array::UnionArrayType<2>>::VARIANTS },
        UnionLayout,
        Buffer,
        OffsetItem,
    >;
}
