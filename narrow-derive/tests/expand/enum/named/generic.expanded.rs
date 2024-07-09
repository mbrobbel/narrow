enum Foo<T> {
    Foo { bar: T },
    Bar(T),
    None,
}
impl<T> ::std::convert::From<&Foo<T>> for ::std::primitive::i8 {
    fn from(value: &Foo<T>) -> ::std::primitive::i8 {
        match *value {
            Foo::Foo { .. } => 0,
            Foo::Bar(..) => 1,
            Foo::None => 2,
        }
    }
}
struct FooVariantFoo<T> {
    bar: T,
}
#[automatically_derived]
impl<T: ::core::default::Default> ::core::default::Default for FooVariantFoo<T> {
    #[inline]
    fn default() -> FooVariantFoo<T> {
        FooVariantFoo {
            bar: ::core::default::Default::default(),
        }
    }
}
struct FooVariantBar<T>(T);
#[automatically_derived]
impl<T: ::core::default::Default> ::core::default::Default for FooVariantBar<T> {
    #[inline]
    fn default() -> FooVariantBar<T> {
        FooVariantBar(::core::default::Default::default())
    }
}
impl<T> narrow::array::union::EnumVariant<0> for Foo<T> {
    type Data = FooVariantFoo<T>;
    fn from_data(value: Self::Data) -> Self {
        Self::Foo { bar: value.bar }
    }
}
impl<T> narrow::array::union::EnumVariant<1> for Foo<T> {
    type Data = FooVariantBar<T>;
    fn from_data(value: Self::Data) -> Self {
        Self::Bar(value.0)
    }
}
impl<T> narrow::array::union::EnumVariant<2> for Foo<T> {
    type Data = ();
    fn from_data(value: Self::Data) -> Self {
        Self::None
    }
}
struct FooArray<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
>(
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>,
);
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
> ::std::default::Default for FooArray<T, Buffer, OffsetItem, UnionLayout>
where
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::default::Default,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::default::Default,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<2>>::Data,
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
> ::std::iter::Extend<Foo<T>>
for FooArray<T, Buffer, OffsetItem, narrow::array::DenseLayout>
where
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<FooVariantFoo<T>>,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<FooVariantBar<T>>,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, narrow::array::DenseLayout>: ::std::iter::Extend<()>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Foo<T>>,
    {
        iter.into_iter()
            .for_each(|variant| {
                match variant {
                    Foo::Foo { bar } => {
                        self.0.extend(::std::iter::once(FooVariantFoo { bar }));
                    }
                    Foo::Bar(_0) => {
                        self.1.extend(::std::iter::once(FooVariantBar(_0)));
                    }
                    Foo::None => {
                        self.2.extend(::std::iter::once(()));
                    }
                }
            });
    }
}
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<Foo<T>>
for FooArray<T, Buffer, OffsetItem, narrow::array::SparseLayout>
where
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<FooVariantFoo<T>>,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<FooVariantBar<T>>,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, narrow::array::SparseLayout>: ::std::iter::Extend<()>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Foo<T>>,
    {
        iter.into_iter()
            .for_each(|variant| {
                match variant {
                    Foo::Foo { bar } => {
                        self.0.extend(::std::iter::once(FooVariantFoo { bar }));
                        self.1
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                        self.2
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                    }
                    Foo::Bar(_0) => {
                        self.1.extend(::std::iter::once(FooVariantBar(_0)));
                        self.0
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                        self.2
                            .extend(
                                ::std::iter::once(::std::default::Default::default()),
                            );
                    }
                    Foo::None => {
                        self.2.extend(::std::iter::once(()));
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
struct FooArrayIntoIter<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
>(
    <<<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout> as ::core::iter::IntoIterator>::IntoIter,
    <<<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout> as ::core::iter::IntoIterator>::IntoIter,
    <<<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout> as ::core::iter::IntoIterator>::IntoIter,
)
where
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::core::iter::IntoIterator,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::core::iter::IntoIterator,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::core::iter::IntoIterator;
impl<
    T: narrow::array::ArrayType,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> narrow::array::union::TypeIdIterator
for FooArrayIntoIter<T, Buffer, OffsetItem, narrow::array::DenseLayout>
where
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::core::iter::IntoIterator<
        Item = <Foo<T> as narrow::array::union::EnumVariant<0>>::Data,
    >,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::core::iter::IntoIterator<
        Item = <Foo<T> as narrow::array::union::EnumVariant<1>>::Data,
    >,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::core::iter::IntoIterator<
        Item = <Foo<T> as narrow::array::union::EnumVariant<2>>::Data,
    >,
{
    type Enum = Foo<T>;
    fn next(
        &mut self,
        type_id: ::std::primitive::i8,
    ) -> ::core::option::Option<Self::Enum> {
        match type_id {
            0 => {
                self.0
                    .next()
                    .map(<Foo<T> as narrow::array::union::EnumVariant<0>>::from_data)
            }
            1 => {
                self.1
                    .next()
                    .map(<Foo<T> as narrow::array::union::EnumVariant<1>>::from_data)
            }
            2 => {
                self.2
                    .next()
                    .map(<Foo<T> as narrow::array::union::EnumVariant<2>>::from_data)
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
for FooArrayIntoIter<T, Buffer, OffsetItem, narrow::array::SparseLayout>
where
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::core::iter::IntoIterator<
        Item = <Foo<T> as narrow::array::union::EnumVariant<0>>::Data,
    >,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::core::iter::IntoIterator<
        Item = <Foo<T> as narrow::array::union::EnumVariant<1>>::Data,
    >,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::core::iter::IntoIterator<
        Item = <Foo<T> as narrow::array::union::EnumVariant<2>>::Data,
    >,
{
    type Enum = Foo<T>;
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
                    .map(<Foo<T> as narrow::array::union::EnumVariant<0>>::from_data)
            }
            1 => {
                self.0.next();
                self.2.next();
                self.1
                    .next()
                    .map(<Foo<T> as narrow::array::union::EnumVariant<1>>::from_data)
            }
            2 => {
                self.0.next();
                self.1.next();
                self.2
                    .next()
                    .map(<Foo<T> as narrow::array::union::EnumVariant<2>>::from_data)
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
for FooArray<T, Buffer, OffsetItem, UnionLayout>
where
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::core::iter::IntoIterator<
        Item = <Foo<T> as narrow::array::union::EnumVariant<0>>::Data,
    >,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::core::iter::IntoIterator<
        Item = <Foo<T> as narrow::array::union::EnumVariant<1>>::Data,
    >,
    <<Foo<
        T,
    > as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <Foo<T> as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::core::iter::IntoIterator<
        Item = <Foo<T> as narrow::array::union::EnumVariant<2>>::Data,
    >,
    FooArrayIntoIter<
        T,
        Buffer,
        OffsetItem,
        UnionLayout,
    >: narrow::array::union::TypeIdIterator,
{
    type VariantIterators = FooArrayIntoIter<T, Buffer, OffsetItem, UnionLayout>;
    fn new_variant_iters(self) -> Self::VariantIterators {
        FooArrayIntoIter::<
            T,
            Buffer,
            OffsetItem,
            UnionLayout,
        >(self.0.into_iter(), self.1.into_iter(), self.2.into_iter())
    }
}
impl<T: narrow::array::ArrayType> narrow::array::UnionArrayType<3> for Foo<T> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = FooArray<T, Buffer, OffsetItem, UnionLayout>;
}
impl<T: narrow::array::ArrayType> narrow::array::ArrayType<Foo<T>> for Foo<T> {
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
#[automatically_derived]
impl<T: ::core::clone::Clone> ::core::clone::Clone for Foo<T> {
    #[inline]
    fn clone(&self) -> Foo<T> {
        match self {
            Foo::Foo { bar: __self_0 } => {
                Foo::Foo {
                    bar: ::core::clone::Clone::clone(__self_0),
                }
            }
            Foo::Bar(__self_0) => Foo::Bar(::core::clone::Clone::clone(__self_0)),
            Foo::None => Foo::None,
        }
    }
}
#[automatically_derived]
impl<T: ::core::marker::Copy> ::core::marker::Copy for Foo<T> {}
