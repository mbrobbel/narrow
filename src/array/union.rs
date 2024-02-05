//! Array for sum types.

use crate::{
    buffer::{BufferType, VecBuffer},
    offset::{self, OffsetElement},
    Length,
};

use super::{Array, ArrayType, Int32Array, Int8Array};

/// Different types of union layouts.
pub trait UnionType {
    /// The array for this union type.
    type Array<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    >
    where
        for<'a> i8: From<&'a T>;
}

/// The dense union layout.
#[derive(Clone, Copy, Debug)]
pub struct DenseLayout;

impl UnionType for DenseLayout {
    type Array<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: OffsetElement> = DenseUnionArray<T, VARIANTS, Buffer, OffsetItem> where for<'a> i8: From<&'a T>;
}

/// The sparse union layout.
#[derive(Clone, Copy, Debug)]
pub struct SparseLayout;

impl UnionType for SparseLayout {
    type Array<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: OffsetElement> = SparseUnionArray<T, VARIANTS, Buffer, OffsetItem> where for<'a> i8: From<&'a T>;
}

/// Indicates that a [`UnionType`] generic is not applicable.
///
/// This is used instead to prevent confusion in code because we don't have default
/// types for generic associated types.
///
/// This still shows up as [`DenseLayout`] in documentation but there is no way
/// to prevent that.
pub type NA = DenseLayout;

/// Union array types.
pub trait UnionArrayType<const VARIANTS: usize>
where
    for<'a> i8: From<&'a Self>,
{
    // can't use this yet in const expressions, unfortunately
    /// The number of variants.
    const VARIANTS: usize = VARIANTS;

    /// The array type storing the variants of the union array.
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>;
}

/// The array data for enum variants stored in union array wrappers.
///
/// Implementations provide the method to convert back to the original enum.
pub trait EnumVariant<const INDEX: usize>: Sized {
    /// The data for this variant. It must be an `ArrayType` because it is stored in an array.
    /// And it must implement `Into<Self>` (this is taking the data and wrapping it in the original enum variant).
    type Data: ArrayType + Default;

    /// Wraps the data in the original enum variant
    fn from_data(value: Self::Data) -> Self;
}

/// Array for sum types.
pub struct UnionArray<
    T: UnionArrayType<VARIANTS>,
    // we need this const here because:
    // generic parameters may not be used in const operations
    // type parameters may not be used in const expressions
    const VARIANTS: usize,
    UnionLayout: UnionType = DenseLayout,
    Buffer: BufferType = VecBuffer,
    OffsetItem: OffsetElement = offset::NA,
>(<UnionLayout as UnionType>::Array<T, VARIANTS, Buffer, OffsetItem>)
where
    for<'a> i8: From<&'a T>;

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        UnionLayout: UnionType,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > Array for UnionArray<T, VARIANTS, UnionLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
{
    type Item = T;
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        UnionLayout: UnionType,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > Length for UnionArray<T, VARIANTS, UnionLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <UnionLayout as UnionType>::Array<T, VARIANTS, Buffer, OffsetItem>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        UnionLayout: UnionType,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > FromIterator<T> for UnionArray<T, VARIANTS, UnionLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    OffsetItem: OffsetElement,
    <UnionLayout as UnionType>::Array<T, VARIANTS, Buffer, OffsetItem>: FromIterator<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// A dense union array.
pub struct DenseUnionArray<
    T: UnionArrayType<VARIANTS>,
    const VARIANTS: usize,
    Buffer: BufferType,
    OffsetItem: OffsetElement,
> where
    for<'a> i8: From<&'a T>,
{
    /// The data for the variants
    pub variants: <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, DenseLayout>,
    /// The types field encodes the variants
    pub types: Int8Array<false, Buffer>,
    /// The offsets in the variant arrays
    pub offsets: Int32Array<false, Buffer>,
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > Length for DenseUnionArray<T, VARIANTS, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
{
    fn len(&self) -> usize {
        self.types.len()
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > FromIterator<T> for DenseUnionArray<T, VARIANTS, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, DenseLayout>: Default + Extend<T>,
    <Buffer as BufferType>::Buffer<i8>: Default + Extend<i8>,
    <Buffer as BufferType>::Buffer<i32>: Default + Extend<i32>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut lens = [0; VARIANTS];
        let ((types, offsets), variants) = iter
            .into_iter()
            .map(|item| {
                let type_id = i8::from(&item);
                let idx = usize::try_from(type_id).expect("bad type id");
                let result = ((type_id, lens[idx]), item);
                lens[idx] += 1;
                result
            })
            .unzip();
        Self {
            variants,
            types,
            offsets,
        }
    }
}

/// A sparse union array.
pub struct SparseUnionArray<
    T: UnionArrayType<VARIANTS>,
    const VARIANTS: usize,
    Buffer: BufferType,
    OffsetItem: OffsetElement,
> where
    for<'a> i8: From<&'a T>,
{
    /// The data for the variants
    pub variants: <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>,
    /// The types field encodes the variants
    pub types: Int8Array<false, Buffer>,
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > Length for SparseUnionArray<T, VARIANTS, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
{
    fn len(&self) -> usize {
        self.types.len()
    }
}

impl<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
    > FromIterator<T> for SparseUnionArray<T, VARIANTS, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>: Default + Extend<T>,
    <Buffer as BufferType>::Buffer<i8>: Default + Extend<i8>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let (types, variants) = iter.into_iter().map(|item| (i8::from(&item), item)).unzip();
        Self { variants, types }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::array::Uint32Array;
    use std::{iter, iter::Extend, marker::PhantomData};

    #[test]
    fn simple() {
        enum Foo {
            Bar(i32),
            Baz(u32),
        }

        struct FooArray<Buffer: BufferType, UnionLayout: UnionType> {
            bar: Int32Array<false, Buffer>,
            baz: Uint32Array<false, Buffer>,
            _ty: PhantomData<UnionLayout>, // we can also use a const generic instead?
        }

        impl<Buffer: BufferType, UnionLayout: UnionType> Default for FooArray<Buffer, UnionLayout>
        where
            Int32Array<false, Buffer>: Default,
            Uint32Array<false, Buffer>: Default,
        {
            fn default() -> Self {
                Self {
                    bar: Int32Array::default(),
                    baz: Uint32Array::default(),
                    _ty: PhantomData,
                }
            }
        }

        impl<Buffer: BufferType> Extend<Foo> for FooArray<Buffer, DenseLayout>
        where
            Int32Array<false, Buffer>: Extend<i32>,
            Uint32Array<false, Buffer>: Extend<u32>,
        {
            fn extend<T: IntoIterator<Item = Foo>>(&mut self, iter: T) {
                iter.into_iter().for_each(|item| match item {
                    Foo::Bar(x) => self.bar.extend(iter::once(x)),
                    Foo::Baz(x) => self.baz.extend(iter::once(x)),
                });
            }
        }

        impl<Buffer: BufferType> Extend<Foo> for FooArray<Buffer, SparseLayout>
        where
            Int32Array<false, Buffer>: Extend<i32>,
            Uint32Array<false, Buffer>: Extend<u32>,
        {
            fn extend<T: IntoIterator<Item = Foo>>(&mut self, iter: T) {
                iter.into_iter().for_each(|item| match item {
                    Foo::Bar(x) => {
                        self.bar.extend(iter::once(x));
                        self.baz.extend(iter::once(Default::default()));
                    }
                    Foo::Baz(x) => {
                        self.baz.extend(iter::once(x));
                        self.bar.extend(iter::once(Default::default()));
                    }
                });
            }
        }

        impl From<&Foo> for i8 {
            fn from(value: &Foo) -> i8 {
                match *value {
                    Foo::Bar(_) => 0,
                    Foo::Baz(_) => 1,
                }
            }
        }

        impl UnionArrayType<2> for Foo {
            type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
                FooArray<Buffer, UnionLayout>;
        }

        let dense_array = [Foo::Bar(0), Foo::Baz(1), Foo::Baz(2), Foo::Bar(3)]
            .into_iter()
            .collect::<UnionArray<Foo, { Foo::VARIANTS }>>();

        assert_eq!(dense_array.0.types.0, [0, 1, 1, 0]);
        assert_eq!(dense_array.0.offsets.0, [0, 0, 1, 1]);
        assert_eq!(dense_array.0.variants.bar.0, [0, 3]);
        assert_eq!(dense_array.0.variants.baz.0, [1, 2]);

        let sparse_array = [Foo::Bar(0), Foo::Baz(1), Foo::Baz(2)]
            .into_iter()
            .collect::<UnionArray<Foo, { Foo::VARIANTS }, SparseLayout>>();

        assert_eq!(sparse_array.0.types.0, [0, 1, 1]);
        assert_eq!(
            sparse_array.0.variants.bar.0,
            [0, i32::default(), i32::default()]
        );
        assert_eq!(sparse_array.0.variants.baz.0, [u32::default(), 1, 2]);
    }

    #[test]
    #[cfg(feature = "derive")]
    #[allow(clippy::too_many_lines)]
    fn with_multiple_fields() {
        use crate::{offset, Length};
        use narrow_derive::ArrayType;

        #[derive(Clone)]
        enum Foo {
            Unit,
            Unnamed(u8, u16),
            Named { a: u32, b: u64 },
        }

        impl ArrayType for Foo {
            type Array<
                Buffer: BufferType,
                OffsetItem: offset::OffsetElement,
                UnionLayout: UnionType,
            > = UnionArray<Foo, { Foo::VARIANTS }, UnionLayout>;
        }

        impl From<&Foo> for i8 {
            fn from(value: &Foo) -> i8 {
                match *value {
                    Foo::Unit => 0,
                    Foo::Unnamed(..) => 1,
                    Foo::Named { .. } => 2,
                }
            }
        }

        impl EnumVariant<0> for Foo {
            type Data = ();

            fn from_data(_value: Self::Data) -> Self {
                Self::Unit
            }
        }

        #[derive(ArrayType, Default)]
        struct FooVariantUnnamed(u8, u16);

        impl EnumVariant<1> for Foo {
            type Data = FooVariantUnnamed;

            fn from_data(value: Self::Data) -> Self {
                Self::Unnamed(value.0, value.1)
            }
        }

        #[derive(ArrayType, Default)]
        struct FooVariantNamed {
            a: u32,
            b: u64,
        }

        impl EnumVariant<2> for Foo {
            type Data = FooVariantNamed;

            fn from_data(value: Self::Data) -> Self {
                Self::Named {
                    a: value.a,
                    b: value.b,
                }
            }
        }

        struct FooArray<Buffer: BufferType, UnionLayout: UnionType> {
            unit: <<Foo as EnumVariant<0>>::Data as ArrayType>::Array<
                Buffer,
                offset::NA,
                UnionLayout,
            >,
            unnamed: <<Foo as EnumVariant<1>>::Data as ArrayType>::Array<
                Buffer,
                offset::NA,
                UnionLayout,
            >,
            named: <<Foo as EnumVariant<2>>::Data as ArrayType>::Array<
                Buffer,
                offset::NA,
                UnionLayout,
            >,
        }

        impl UnionArrayType<3> for Foo {
            type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
                FooArray<Buffer, UnionLayout>;
        }

        impl<Buffer: BufferType, UnionLayout: UnionType> Default for FooArray<Buffer, UnionLayout>
        where
            <<Foo as EnumVariant<0>>::Data as ArrayType>::Array<Buffer, offset::NA, UnionLayout>:
                Default,
            <<Foo as EnumVariant<1>>::Data as ArrayType>::Array<Buffer, offset::NA, UnionLayout>:
                Default,
            <<Foo as EnumVariant<2>>::Data as ArrayType>::Array<Buffer, offset::NA, UnionLayout>:
                Default,
        {
            fn default() -> Self {
                #[allow(clippy::default_trait_access)]
                Self {
                    unit: Default::default(),
                    unnamed: Default::default(),
                    named: Default::default(),
                }
            }
        }

        impl<Buffer: BufferType> Extend<Foo> for FooArray<Buffer, DenseLayout>
        where
            <<Foo as EnumVariant<0>>::Data as ArrayType>::Array<Buffer, offset::NA, DenseLayout>:
                Extend<()>,
            <<Foo as EnumVariant<1>>::Data as ArrayType>::Array<Buffer, offset::NA, DenseLayout>:
                Extend<FooVariantUnnamed>,
            <<Foo as EnumVariant<2>>::Data as ArrayType>::Array<Buffer, offset::NA, DenseLayout>:
                Extend<FooVariantNamed>,
        {
            fn extend<T: IntoIterator<Item = Foo>>(&mut self, iter: T) {
                iter.into_iter().for_each(|item| match item {
                    Foo::Unit => self.unit.extend(iter::once(())),
                    Foo::Unnamed(a, b) => self.unnamed.extend(iter::once(FooVariantUnnamed(a, b))),
                    Foo::Named { a, b } => self.named.extend(iter::once(FooVariantNamed { a, b })),
                });
            }
        }

        impl<Buffer: BufferType> Extend<Foo> for FooArray<Buffer, SparseLayout>
        where
            <<Foo as EnumVariant<0>>::Data as ArrayType>::Array<Buffer, offset::NA, DenseLayout>:
                Extend<()>,
            <<Foo as EnumVariant<1>>::Data as ArrayType>::Array<Buffer, offset::NA, DenseLayout>:
                Extend<FooVariantUnnamed>,
            <<Foo as EnumVariant<2>>::Data as ArrayType>::Array<Buffer, offset::NA, DenseLayout>:
                Extend<FooVariantNamed>,
        {
            fn extend<T: IntoIterator<Item = Foo>>(&mut self, iter: T) {
                iter.into_iter().for_each(|item| match item {
                    Foo::Unit => {
                        self.unit.extend(iter::once(()));
                        self.unnamed
                            .extend(iter::once(FooVariantUnnamed::default()));
                        self.named.extend(iter::once(FooVariantNamed::default()));
                    }
                    Foo::Unnamed(a, b) => {
                        self.unit.extend(iter::once(()));
                        self.unnamed.extend(iter::once(FooVariantUnnamed(a, b)));
                        self.named.extend(iter::once(FooVariantNamed::default()));
                    }
                    Foo::Named { a, b } => {
                        self.unit.extend(iter::once(()));
                        self.unnamed
                            .extend(iter::once(FooVariantUnnamed::default()));
                        self.named.extend(iter::once(FooVariantNamed { a, b }));
                    }
                });
            }
        }

        let input = [Foo::Unit, Foo::Unnamed(1, 2), Foo::Named { a: 3, b: 4 }];
        let dense_array = input
            .into_iter()
            .collect::<UnionArray<Foo, { Foo::VARIANTS }>>();

        assert_eq!(dense_array.0.types.0, [0, 1, 2]);
        assert_eq!(dense_array.0.offsets.0, [0, 0, 0]);
        assert_eq!(dense_array.0.variants.unit.0.len(), 1);
        assert_eq!(dense_array.0.variants.unnamed.0 .0 .0, [1]);
        assert_eq!(dense_array.0.variants.unnamed.0 .1 .0, [2]);
        assert_eq!(dense_array.0.variants.named.0.a.0, [3]);
        assert_eq!(dense_array.0.variants.named.0.b.0, [4]);
    }

    #[test]
    #[cfg(feature = "derive")]
    fn derive() {
        use crate::ArrayType;

        #[derive(ArrayType, Clone, Copy)]
        enum Test {
            Foo { bar: u8 },
            Bar(bool),
            None,
        }
        let input = [
            Test::None,
            Test::Bar(true),
            Test::Foo { bar: 123 },
            Test::None,
        ];
        let dense_array = input
            .into_iter()
            .collect::<UnionArray<Test, { Test::VARIANTS }>>();
        assert_eq!(dense_array.len(), 4);
        assert_eq!(dense_array.0.types.0, &[2, 1, 0, 2]);
        assert_eq!(dense_array.0.offsets.0, &[0, 0, 0, 1]);
        assert_eq!(dense_array.0.variants.0 .0.bar.0, &[123]);
        assert_eq!(
            dense_array
                .0
                .variants
                .1
                 .0
                 .0
                .into_iter()
                .collect::<Vec<_>>(),
            &[true]
        );
        assert_eq!(dense_array.0.variants.2 .0.len(), 2);

        let sparse_array = input
            .into_iter()
            .collect::<UnionArray<Test, { Test::VARIANTS }, SparseLayout>>();
        assert_eq!(sparse_array.len(), 4);
        assert_eq!(sparse_array.0.types.0, &[2, 1, 0, 2]);
        assert_eq!(sparse_array.0.variants.0 .0.bar.0, &[0, 0, 123, 0]);
        assert_eq!(
            sparse_array
                .0
                .variants
                .1
                 .0
                 .0
                .into_iter()
                .collect::<Vec<_>>(),
            &[false, true, false, false]
        );
        assert_eq!(sparse_array.0.variants.2 .0.len(), 4);
    }
}
