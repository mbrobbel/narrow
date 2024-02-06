enum FooBar<T>
where
    T: Default,
    Self: Clone,
{
    Foo,
    Bar(T),
    FooBar { foo_bar: T },
}
impl<T> ::std::convert::From<&FooBar<T>> for ::std::primitive::i8
where
    T: Default,
    Self: Clone,
{
    fn from(value: &FooBar<T>) -> i8 {
        match *value {
            FooBar::Foo => 0,
            FooBar::Bar(..) => 1,
            FooBar::FooBar { .. } => 2,
        }
    }
}
struct FooBarVariantBar<T>(
    T,
)
where
    T: Default;
#[automatically_derived]
impl<T: ::core::default::Default> ::core::default::Default for FooBarVariantBar<T>
where
    T: Default,
{
    #[inline]
    fn default() -> FooBarVariantBar<T> {
        FooBarVariantBar(::core::default::Default::default())
    }
}
struct FooBarVariantFooBar<T>
where
    T: Default,
{
    foo_bar: T,
}
#[automatically_derived]
impl<T: ::core::default::Default> ::core::default::Default for FooBarVariantFooBar<T>
where
    T: Default,
{
    #[inline]
    fn default() -> FooBarVariantFooBar<T> {
        FooBarVariantFooBar {
            foo_bar: ::core::default::Default::default(),
        }
    }
}
impl<T> narrow::array::union::EnumVariant<0> for FooBar<T>
where
    T: Default,
    Self: Clone,
{
    type Data = ();
    fn from_data(value: Self::Data) -> Self {
        Self::Foo
    }
}
impl<T> narrow::array::union::EnumVariant<1> for FooBar<T>
where
    T: Default,
    Self: Clone,
{
    type Data = FooBarVariantBar<T>;
    fn from_data(value: Self::Data) -> Self {
        Self::Bar(value.0)
    }
}
impl<T> narrow::array::union::EnumVariant<2> for FooBar<T>
where
    T: Default,
    Self: Clone,
{
    type Data = FooBarVariantFooBar<T>;
    fn from_data(value: Self::Data) -> Self {
        Self::FooBar {
            foo_bar: value.foo_bar,
        }
    }
}
struct FooBarArray<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
>(
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType>::Array<Buffer, OffsetItem, UnionLayout>,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType>::Array<Buffer, OffsetItem, UnionLayout>,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType>::Array<Buffer, OffsetItem, UnionLayout>,
)
where
    T: Default,
    FooBar<T>: Clone;
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
> ::std::default::Default for FooBarArray<T, Buffer, OffsetItem, UnionLayout>
where
    T: Default,
    FooBar<T>: Clone,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::std::default::Default,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::std::default::Default,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        2,
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
        )
    }
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<FooBar<T>> for FooBarArray<T, Buffer, OffsetItem, DenseLayout>
where
    T: Default,
    FooBar<T>: Clone,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<()>,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<FooBarVariantBar<T>>,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType>::Array<
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
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<FooBar<T>> for FooBarArray<T, Buffer, OffsetItem, SparseLayout>
where
    T: Default,
    FooBar<T>: Clone,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<()>,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<FooBarVariantBar<T>>,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType>::Array<
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
impl<T: narrow::array::ArrayType> narrow::array::UnionArrayType<3> for FooBar<T>
where
    T: Default,
    Self: Clone,
{
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = FooBarArray<T, Buffer, OffsetItem, UnionLayout>;
}
impl<T: narrow::array::ArrayType> narrow::array::ArrayType for FooBar<T>
where
    T: Default,
    Self: Clone,
{
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
