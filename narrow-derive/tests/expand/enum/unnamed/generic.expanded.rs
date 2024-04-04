enum FooBar<T: Default> {
    Foo,
    Bar(T),
    FooBar { foo_bar: T },
}
impl<T: Default> ::std::convert::From<&FooBar<T>> for ::std::primitive::i8 {
    fn from(value: &FooBar<T>) -> ::std::primitive::i8 {
        match *value {
            FooBar::Foo => 0,
            FooBar::Bar(..) => 1,
            FooBar::FooBar { .. } => 2,
        }
    }
}
struct FooBarVariantBar<T: Default>(T);
#[automatically_derived]
impl<T: ::core::default::Default + Default> ::core::default::Default
for FooBarVariantBar<T> {
    #[inline]
    fn default() -> FooBarVariantBar<T> {
        FooBarVariantBar(::core::default::Default::default())
    }
}
struct FooBarVariantFooBar<T: Default> {
    foo_bar: T,
}
#[automatically_derived]
impl<T: ::core::default::Default + Default> ::core::default::Default
for FooBarVariantFooBar<T> {
    #[inline]
    fn default() -> FooBarVariantFooBar<T> {
        FooBarVariantFooBar {
            foo_bar: ::core::default::Default::default(),
        }
    }
}
impl<T: Default> narrow::array::union::EnumVariant<0> for FooBar<T> {
    type Data = ();
    fn from_data(value: Self::Data) -> Self {
        Self::Foo
    }
}
impl<T: Default> narrow::array::union::EnumVariant<1> for FooBar<T> {
    type Data = FooBarVariantBar<T>;
    fn from_data(value: Self::Data) -> Self {
        Self::Bar(value.0)
    }
}
impl<T: Default> narrow::array::union::EnumVariant<2> for FooBar<T> {
    type Data = FooBarVariantFooBar<T>;
    fn from_data(value: Self::Data) -> Self {
        Self::FooBar {
            foo_bar: value.foo_bar,
        }
    }
}
struct FooBarArray<
    T: Default + narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
>(
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>,
);
impl<
    T: Default + narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
> ::std::default::Default for FooBarArray<T, Buffer, OffsetItem, UnionLayout>
where
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::default::Default,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::default::Default,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::default::Default,
{
    fn default() -> Self {
        Self(
            ::std::default::Default::default(),
            ::std::default::Default::default(),
            ::std::default::Default::default(),
        )
    }
}
impl<
    T: Default + narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<FooBar<T>>
for FooBarArray<T, Buffer, OffsetItem, narrow::array::DenseLayout>
where
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, narrow::array::DenseLayout>: ::std::iter::Extend<()>,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<FooBarVariantBar<T>>,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<FooBarVariantFooBar<T>>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = FooBar<T>>,
    {
        iter.into_iter()
            .for_each(|variant| {
                match variant {
                    FooBar::Foo => {
                        self.0.extend(::std::iter::once(()));
                    }
                    FooBar::Bar(_0) => {
                        self.1.extend(::std::iter::once(FooBarVariantBar(_0)));
                    }
                    FooBar::FooBar { foo_bar } => {
                        self.2
                            .extend(::std::iter::once(FooBarVariantFooBar { foo_bar }));
                    }
                }
            });
    }
}
impl<
    T: Default + narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<FooBar<T>>
for FooBarArray<T, Buffer, OffsetItem, narrow::array::SparseLayout>
where
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, narrow::array::SparseLayout>: ::std::iter::Extend<()>,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<FooBarVariantBar<T>>,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<FooBarVariantFooBar<T>>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = FooBar<T>>,
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
                    }
                    FooBar::Bar(_0) => {
                        self.1.extend(::std::iter::once(FooBarVariantBar(_0)));
                        self.0
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                        self.2
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                    }
                    FooBar::FooBar { foo_bar } => {
                        self.2
                            .extend(::std::iter::once(FooBarVariantFooBar { foo_bar }));
                        self.0
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                        self.1
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                    }
                }
            });
    }
}
impl<T: Default + narrow::array::ArrayType> narrow::array::UnionArrayType<3>
for FooBar<T> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = FooBarArray<T, Buffer, OffsetItem, UnionLayout>;
}
impl<T: Default + narrow::array::ArrayType> narrow::array::ArrayType<FooBar<T>>
for FooBar<T> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::UnionArray<
        Self,
        { <Self as narrow::array::UnionArrayType<3>>::VARIANTS },
        UnionLayout,
        Buffer,
    >;
}
