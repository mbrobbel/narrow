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
    fn from(value: &FooBar<T>) -> ::std::primitive::i8 {
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
)
where
    T: Default,
    FooBar<T>: Clone;
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
> ::std::clone::Clone for FooBarArray<T, Buffer, OffsetItem, UnionLayout>
where
    T: Default,
    FooBar<T>: Clone,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::clone::Clone,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::clone::Clone,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), self.2.clone())
    }
}
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
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<FooBar<T>>
for FooBarArray<T, Buffer, OffsetItem, narrow::array::DenseLayout>
where
    T: Default,
    FooBar<T>: Clone,
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
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<FooBar<T>>
for FooBarArray<T, Buffer, OffsetItem, narrow::array::SparseLayout>
where
    T: Default,
    FooBar<T>: Clone,
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
struct FooBarArrayIntoIter<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
>(
    <<<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout> as ::core::iter::IntoIterator>::IntoIter,
    <<<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout> as ::core::iter::IntoIterator>::IntoIter,
    <<<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout> as ::core::iter::IntoIterator>::IntoIter,
)
where
    T: Default,
    FooBar<T>: Clone,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::core::iter::IntoIterator,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::core::iter::IntoIterator,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::core::iter::IntoIterator;
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> narrow::array::union::TypeIdIterator
for FooBarArrayIntoIter<T, Buffer, OffsetItem, narrow::array::DenseLayout>
where
    T: Default,
    FooBar<T>: Clone,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar<T> as narrow::array::union::EnumVariant<0>>::Data,
    >,
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
    >: ::core::iter::IntoIterator<
        Item = <FooBar<T> as narrow::array::union::EnumVariant<1>>::Data,
    >,
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
    >: ::core::iter::IntoIterator<
        Item = <FooBar<T> as narrow::array::union::EnumVariant<2>>::Data,
    >,
{
    type Enum = FooBar<T>;
    fn next(
        &mut self,
        type_id: ::std::primitive::i8,
    ) -> ::core::option::Option<Self::Enum> {
        match type_id {
            0 => {
                self.0
                    .next()
                    .map(<FooBar<T> as narrow::array::union::EnumVariant<0>>::from_data)
            }
            1 => {
                self.1
                    .next()
                    .map(<FooBar<T> as narrow::array::union::EnumVariant<1>>::from_data)
            }
            2 => {
                self.2
                    .next()
                    .map(<FooBar<T> as narrow::array::union::EnumVariant<2>>::from_data)
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
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> narrow::array::union::TypeIdIterator
for FooBarArrayIntoIter<T, Buffer, OffsetItem, narrow::array::SparseLayout>
where
    T: Default,
    FooBar<T>: Clone,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar<T> as narrow::array::union::EnumVariant<0>>::Data,
    >,
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
    >: ::core::iter::IntoIterator<
        Item = <FooBar<T> as narrow::array::union::EnumVariant<1>>::Data,
    >,
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
    >: ::core::iter::IntoIterator<
        Item = <FooBar<T> as narrow::array::union::EnumVariant<2>>::Data,
    >,
{
    type Enum = FooBar<T>;
    fn next(
        &mut self,
        type_id: ::std::primitive::i8,
    ) -> ::core::option::Option<Self::Enum> {
        match type_id {
            0 => {
                self.1.next();
                self.2.next();
                self.0
                    .next()
                    .map(<FooBar<T> as narrow::array::union::EnumVariant<0>>::from_data)
            }
            1 => {
                self.0.next();
                self.2.next();
                self.1
                    .next()
                    .map(<FooBar<T> as narrow::array::union::EnumVariant<1>>::from_data)
            }
            2 => {
                self.0.next();
                self.1.next();
                self.2
                    .next()
                    .map(<FooBar<T> as narrow::array::union::EnumVariant<2>>::from_data)
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
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
> narrow::array::union::UnionArrayIterators
for FooBarArray<T, Buffer, OffsetItem, UnionLayout>
where
    T: Default,
    FooBar<T>: Clone,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar<T> as narrow::array::union::EnumVariant<0>>::Data,
    >,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar<T> as narrow::array::union::EnumVariant<1>>::Data,
    >,
    <<FooBar<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar<T> as narrow::array::union::EnumVariant<2>>::Data,
    >,
    FooBarArrayIntoIter<
        T,
        Buffer,
        OffsetItem,
        UnionLayout,
    >: narrow::array::union::TypeIdIterator,
{
    type VariantIterators = FooBarArrayIntoIter<T, Buffer, OffsetItem, UnionLayout>;
    fn new_variant_iters(self) -> Self::VariantIterators {
        FooBarArrayIntoIter::<
            T,
            Buffer,
            OffsetItem,
            UnionLayout,
        >(self.0.into_iter(), self.1.into_iter(), self.2.into_iter())
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
impl<T: narrow::array::ArrayType> narrow::array::ArrayType<FooBar<T>> for FooBar<T>
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
        OffsetItem,
    >;
}
