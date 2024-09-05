enum FooBar<const X: bool> {
    Foo,
    Bar,
}
impl<const X: bool> ::std::convert::From<&FooBar<X>> for ::std::primitive::i8 {
    fn from(value: &FooBar<X>) -> ::std::primitive::i8 {
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
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>,
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>,
);
impl<
    const X: bool,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
> ::std::clone::Clone for FooBarArray<X, Buffer, OffsetItem, UnionLayout>
where
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::clone::Clone,
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}
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
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::default::Default,
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::default::Default,
{
    fn default() -> Self {
        Self(::std::default::Default::default(), ::std::default::Default::default())
    }
}
impl<
    const X: bool,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> ::std::iter::Extend<FooBar<X>>
for FooBarArray<X, Buffer, OffsetItem, narrow::array::DenseLayout>
where
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, narrow::array::DenseLayout>: ::std::iter::Extend<()>,
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, narrow::array::DenseLayout>: ::std::iter::Extend<()>,
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
> ::std::iter::Extend<FooBar<X>>
for FooBarArray<X, Buffer, OffsetItem, narrow::array::SparseLayout>
where
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, narrow::array::SparseLayout>: ::std::iter::Extend<()>,
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, narrow::array::SparseLayout>: ::std::iter::Extend<()>,
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
struct FooBarArrayIntoIter<
    const X: bool,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
>(
    <<<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout> as ::core::iter::IntoIterator>::IntoIter,
    <<<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout> as ::core::iter::IntoIterator>::IntoIter,
)
where
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::core::iter::IntoIterator,
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::core::iter::IntoIterator;
impl<
    const X: bool,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> narrow::array::union::TypeIdIterator
for FooBarArrayIntoIter<X, Buffer, OffsetItem, narrow::array::DenseLayout>
where
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar<X> as narrow::array::union::EnumVariant<0>>::Data,
    >,
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar<X> as narrow::array::union::EnumVariant<1>>::Data,
    >,
{
    type Enum = FooBar<X>;
    fn next(
        &mut self,
        type_id: ::std::primitive::i8,
    ) -> ::core::option::Option<Self::Enum> {
        match type_id {
            0 => {
                self.0
                    .next()
                    .map(<FooBar<X> as narrow::array::union::EnumVariant<0>>::from_data)
            }
            1 => {
                self.1
                    .next()
                    .map(<FooBar<X> as narrow::array::union::EnumVariant<1>>::from_data)
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
    const X: bool,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
> narrow::array::union::TypeIdIterator
for FooBarArrayIntoIter<X, Buffer, OffsetItem, narrow::array::SparseLayout>
where
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar<X> as narrow::array::union::EnumVariant<0>>::Data,
    >,
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar<X> as narrow::array::union::EnumVariant<1>>::Data,
    >,
{
    type Enum = FooBar<X>;
    fn next(
        &mut self,
        type_id: ::std::primitive::i8,
    ) -> ::core::option::Option<Self::Enum> {
        match type_id {
            0 => {
                self.1.next();
                self.0
                    .next()
                    .map(<FooBar<X> as narrow::array::union::EnumVariant<0>>::from_data)
            }
            1 => {
                self.0.next();
                self.1
                    .next()
                    .map(<FooBar<X> as narrow::array::union::EnumVariant<1>>::from_data)
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
    const X: bool,
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::OffsetElement,
    UnionLayout: narrow::array::UnionType,
> narrow::array::union::UnionArrayIterators
for FooBarArray<X, Buffer, OffsetItem, UnionLayout>
where
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar<X> as narrow::array::union::EnumVariant<0>>::Data,
    >,
    <<FooBar<
        X,
    > as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar<X> as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar<X> as narrow::array::union::EnumVariant<1>>::Data,
    >,
    FooBarArrayIntoIter<
        X,
        Buffer,
        OffsetItem,
        UnionLayout,
    >: narrow::array::union::TypeIdIterator,
{
    type VariantIterators = FooBarArrayIntoIter<X, Buffer, OffsetItem, UnionLayout>;
    fn new_variant_iters(self) -> Self::VariantIterators {
        FooBarArrayIntoIter::<
            X,
            Buffer,
            OffsetItem,
            UnionLayout,
        >(self.0.into_iter(), self.1.into_iter())
    }
}
impl<const X: bool> narrow::array::UnionArrayType<2> for FooBar<X> {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::OffsetElement,
        UnionLayout: narrow::array::UnionType,
    > = FooBarArray<X, Buffer, OffsetItem, UnionLayout>;
}
impl<const X: bool> narrow::array::ArrayType<FooBar<X>> for FooBar<X> {
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
