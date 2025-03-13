enum FooBar {
    Unit,
    Foo { bar: u32 },
    Bar { foo: bool },
    FooBar { foo: String, bar: Option<u8> },
}
impl ::std::convert::From<&FooBar> for ::std::primitive::i8 {
    fn from(value: &FooBar) -> ::std::primitive::i8 {
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
    OffsetItem: narrow::offset::Offset,
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
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<3>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>,
);
impl<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::Offset,
    UnionLayout: narrow::array::UnionType,
> ::std::clone::Clone for FooBarArray<Buffer, OffsetItem, UnionLayout>
where
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::clone::Clone,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::clone::Clone,
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::clone::Clone,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<3>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), self.2.clone(), self.3.clone())
    }
}
impl<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::Offset,
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
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::default::Default,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<3>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::std::default::Default,
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
    OffsetItem: narrow::offset::Offset,
> narrow::array::union::DenseOffset
for FooBarArray<Buffer, OffsetItem, narrow::array::DenseLayout> {
    fn variant_len(&self, type_id: i8) -> usize {
        match type_id {
            0 => self.0.len(),
            1 => self.1.len(),
            2 => self.2.len(),
            3 => self.3.len(),
            _ => {
                ::core::panicking::panic_fmt(format_args!("bad type id"));
            }
        }
    }
}
impl<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::Offset,
> ::std::iter::Extend<FooBar>
for FooBarArray<Buffer, OffsetItem, narrow::array::DenseLayout>
where
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, narrow::array::DenseLayout>: ::std::iter::Extend<()>,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<FooBarVariantFoo>,
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::std::iter::Extend<FooBarVariantBar>,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<3>>::Data,
    >>::Array<
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
    OffsetItem: narrow::offset::Offset,
> ::std::iter::Extend<FooBar>
for FooBarArray<Buffer, OffsetItem, narrow::array::SparseLayout>
where
    <<FooBar as narrow::array::union::EnumVariant<
        0,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<0>>::Data,
    >>::Array<Buffer, OffsetItem, narrow::array::SparseLayout>: ::std::iter::Extend<()>,
    <<FooBar as narrow::array::union::EnumVariant<
        1,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<1>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<FooBarVariantFoo>,
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::std::iter::Extend<FooBarVariantBar>,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<3>>::Data,
    >>::Array<
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
struct FooBarArrayIntoIter<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::Offset,
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
    <<<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout> as ::core::iter::IntoIterator>::IntoIter,
    <<<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<3>>::Data,
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
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::core::iter::IntoIterator,
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::core::iter::IntoIterator,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<3>>::Data,
    >>::Array<Buffer, OffsetItem, UnionLayout>: ::core::iter::IntoIterator;
impl<
    Buffer: narrow::buffer::BufferType,
    OffsetItem: narrow::offset::Offset,
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
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar as narrow::array::union::EnumVariant<2>>::Data,
    >,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<3>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::DenseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar as narrow::array::union::EnumVariant<3>>::Data,
    >,
{
    type Enum = FooBar;
    fn next(
        &mut self,
        type_id: ::std::primitive::i8,
    ) -> ::core::option::Option<Self::Enum> {
        match type_id {
            0 => {
                self.0
                    .next()
                    .map(<FooBar as narrow::array::union::EnumVariant<0>>::from_data)
            }
            1 => {
                self.1
                    .next()
                    .map(<FooBar as narrow::array::union::EnumVariant<1>>::from_data)
            }
            2 => {
                self.2
                    .next()
                    .map(<FooBar as narrow::array::union::EnumVariant<2>>::from_data)
            }
            3 => {
                self.3
                    .next()
                    .map(<FooBar as narrow::array::union::EnumVariant<3>>::from_data)
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
    OffsetItem: narrow::offset::Offset,
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
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar as narrow::array::union::EnumVariant<2>>::Data,
    >,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<3>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        narrow::array::SparseLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar as narrow::array::union::EnumVariant<3>>::Data,
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
                self.2.next();
                self.3.next();
                self.0
                    .next()
                    .map(<FooBar as narrow::array::union::EnumVariant<0>>::from_data)
            }
            1 => {
                self.0.next();
                self.2.next();
                self.3.next();
                self.1
                    .next()
                    .map(<FooBar as narrow::array::union::EnumVariant<1>>::from_data)
            }
            2 => {
                self.0.next();
                self.1.next();
                self.3.next();
                self.2
                    .next()
                    .map(<FooBar as narrow::array::union::EnumVariant<2>>::from_data)
            }
            3 => {
                self.0.next();
                self.1.next();
                self.2.next();
                self.3
                    .next()
                    .map(<FooBar as narrow::array::union::EnumVariant<3>>::from_data)
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
    OffsetItem: narrow::offset::Offset,
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
    <<FooBar as narrow::array::union::EnumVariant<
        2,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<2>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar as narrow::array::union::EnumVariant<2>>::Data,
    >,
    <<FooBar as narrow::array::union::EnumVariant<
        3,
    >>::Data as narrow::array::ArrayType<
        <FooBar as narrow::array::union::EnumVariant<3>>::Data,
    >>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: ::core::iter::IntoIterator<
        Item = <FooBar as narrow::array::union::EnumVariant<3>>::Data,
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
        >(self.0.into_iter(), self.1.into_iter(), self.2.into_iter(), self.3.into_iter())
    }
}
impl narrow::array::UnionArrayType<4> for FooBar {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = FooBarArray<Buffer, OffsetItem, UnionLayout>;
}
impl narrow::array::ArrayType<FooBar> for FooBar {
    type Array<
        Buffer: narrow::buffer::BufferType,
        OffsetItem: narrow::offset::Offset,
        UnionLayout: narrow::array::UnionType,
    > = narrow::array::UnionArray<
        Self,
        { <Self as narrow::array::UnionArrayType<4>>::VARIANTS },
        UnionLayout,
        Buffer,
        OffsetItem,
    >;
}
