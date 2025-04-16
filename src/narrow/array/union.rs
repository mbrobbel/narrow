//! Array for sum types.

use std::iter;

use crate::{
    Length, NonNullable,
    buffer::{BufferType, VecBuffer},
    offset::{self, Offset},
};

use super::{Array, ArrayType, Int8Array, Int32Array};

/// Different types of union layouts.
pub trait UnionType {
    /// The array for this union type.
    type Array<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: Offset,
    >
    where
        for<'a> i8: From<&'a T>;
}

/// The dense union layout.
#[derive(Clone, Copy, Debug)]
pub struct DenseLayout;

impl UnionType for DenseLayout {
    type Array<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: Offset,
    >
        = DenseUnionArray<T, VARIANTS, Buffer, OffsetItem>
    where
        for<'a> i8: From<&'a T>;
}

/// The sparse union layout.
#[derive(Clone, Copy, Debug)]
pub struct SparseLayout;

impl UnionType for SparseLayout {
    type Array<
        T: UnionArrayType<VARIANTS>,
        const VARIANTS: usize,
        Buffer: BufferType,
        OffsetItem: Offset,
    >
        = SparseUnionArray<T, VARIANTS, Buffer, OffsetItem>
    where
        for<'a> i8: From<&'a T>;
}

/// Indicates that a [`UnionType`] generic is not applicable.
///
/// This is used instead to prevent confusion in code because we don't have default
/// types for generic associated types.
///
/// This still shows up as [`SparseLayout`] in documentation but there is no way
/// to prevent that.
pub type NA = SparseLayout;

/// Union array types.
pub trait UnionArrayType<const VARIANTS: usize>
where
    for<'a> i8: From<&'a Self>,
{
    // can't use this yet in const expressions, unfortunately
    /// The number of variants.
    const VARIANTS: usize = VARIANTS;

    /// The array type storing the variants of the union array.
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType>;
}

/// The array data for enum variants stored in union array wrappers.
///
/// Implementations provide the method to convert back to the original enum.
pub trait EnumVariant<const INDEX: usize>: Sized {
    /// The data for this variant. It must be an `ArrayType` because it is stored in an array.
    /// And it must implement `Into<Self>` (this is taking the data and wrapping it in the original enum variant).
    type Data: ArrayType<Self::Data> + Default;

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
    OffsetItem: Offset = offset::NA,
>(pub(crate) UnionLayout::Array<T, VARIANTS, Buffer, OffsetItem>)
where
    for<'a> i8: From<&'a T>;

impl<
    T: UnionArrayType<VARIANTS>,
    const VARIANTS: usize,
    UnionLayout: UnionType,
    Buffer: BufferType,
    OffsetItem: Offset,
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
    OffsetItem: Offset,
> Clone for UnionArray<T, VARIANTS, UnionLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    UnionLayout::Array<T, VARIANTS, Buffer, OffsetItem>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<
    T: UnionArrayType<VARIANTS>,
    const VARIANTS: usize,
    UnionLayout: UnionType,
    Buffer: BufferType,
    OffsetItem: Offset,
> Default for UnionArray<T, VARIANTS, UnionLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    UnionLayout::Array<T, VARIANTS, Buffer, OffsetItem>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<
    T: UnionArrayType<VARIANTS>,
    const VARIANTS: usize,
    UnionLayout: UnionType,
    Buffer: BufferType,
    OffsetItem: Offset,
> Extend<T> for UnionArray<T, VARIANTS, UnionLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <UnionLayout as UnionType>::Array<T, VARIANTS, Buffer, OffsetItem>: Extend<T>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<
    T: UnionArrayType<VARIANTS>,
    const VARIANTS: usize,
    UnionLayout: UnionType,
    Buffer: BufferType,
    OffsetItem: Offset,
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
    OffsetItem: Offset,
> FromIterator<T> for UnionArray<T, VARIANTS, UnionLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    OffsetItem: Offset,
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
    Buffer: BufferType = VecBuffer,
    OffsetItem: Offset = i32,
> where
    for<'a> i8: From<&'a T>,
{
    /// The data for the variants
    pub variants: <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, DenseLayout>,
    /// The types field encodes the variants
    pub types: Int8Array<NonNullable, Buffer>,
    /// The offsets in the variant arrays
    pub offsets: Int32Array<NonNullable, Buffer>,
}

/// A trait that should be implemented by the derive macro for dense layout
/// union arrays.
#[doc(hidden)]
pub trait DenseOffset {
    /// Returns the length (number of items) of the variant with the given type
    /// id stored in this array.
    fn variant_len(&self, type_id: i8) -> usize;
}

impl<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: Offset>
    Clone for DenseUnionArray<T, VARIANTS, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, DenseLayout>: Clone,
    Int8Array<NonNullable, Buffer>: Clone,
    Int32Array<NonNullable, Buffer>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            variants: self.variants.clone(),
            types: self.types.clone(),
            offsets: self.offsets.clone(),
        }
    }
}

impl<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: Offset>
    Default for DenseUnionArray<T, VARIANTS, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, DenseLayout>: Default,
    Int8Array<NonNullable, Buffer>: Default,
    Int32Array<NonNullable, Buffer>: Default,
{
    fn default() -> Self {
        Self {
            variants: Default::default(),
            types: Int8Array::default(),
            offsets: Int32Array::default(),
        }
    }
}

impl<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: Offset>
    Length for DenseUnionArray<T, VARIANTS, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
{
    fn len(&self) -> usize {
        self.types.len()
    }
}

impl<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: Offset>
    Extend<T> for DenseUnionArray<T, VARIANTS, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, DenseLayout>:
        Extend<T> + DenseOffset,
    Int8Array<NonNullable, Buffer>: Extend<i8>,
    Int32Array<NonNullable, Buffer>: Extend<i32>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|item| {
            let type_id = i8::from(&item);
            let idx = usize::try_from(type_id).expect("bad type id");
            assert!(idx < VARIANTS, "type id greater than number of variants");

            // For dense unions, we need to track the current offset for each variant
            // Count the current elements of this type
            let offset = self.variants.variant_len(type_id);

            self.types.extend(iter::once(type_id));
            self.offsets
                .extend(iter::once(i32::try_from(offset).expect("overflow")));
            self.variants.extend(iter::once(item));
        });
    }
}

impl<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: Offset>
    FromIterator<T> for DenseUnionArray<T, VARIANTS, Buffer, OffsetItem>
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

                assert!(idx < VARIANTS, "type id greater than number of variants");
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
    Buffer: BufferType = VecBuffer,
    OffsetItem: Offset = i32,
> where
    for<'a> i8: From<&'a T>,
{
    /// The data for the variants
    pub variants: <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>,
    /// The types field encodes the variants
    pub types: Int8Array<NonNullable, Buffer>,
}

impl<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: Offset>
    Clone for SparseUnionArray<T, VARIANTS, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>: Clone,
    Int8Array<NonNullable, Buffer>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            variants: self.variants.clone(),
            types: self.types.clone(),
        }
    }
}

impl<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: Offset>
    Default for SparseUnionArray<T, VARIANTS, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>: Default,
    Int8Array<NonNullable, Buffer>: Default,
{
    fn default() -> Self {
        Self {
            variants: Default::default(),
            types: Int8Array::default(),
        }
    }
}

impl<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: Offset>
    Extend<T> for SparseUnionArray<T, VARIANTS, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>: Extend<T>,
    Int8Array<NonNullable, Buffer>: Extend<i8>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|item| {
            self.types.extend(iter::once(i8::from(&item)));
            self.variants.extend(iter::once(item));
        });
    }
}

impl<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: Offset>
    Length for SparseUnionArray<T, VARIANTS, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
{
    fn len(&self) -> usize {
        self.types.len()
    }
}

impl<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: Offset>
    FromIterator<T> for SparseUnionArray<T, VARIANTS, Buffer, OffsetItem>
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

/// Types that return a constructed `enum` by advancing
/// iterator(s) of variants of a union array given the
/// `type_id` for the variant.
pub trait TypeIdIterator {
    /// The rust enum
    type Enum;

    /// Advances the variant iterator(s) to get the next
    /// value for the `type_id`.
    fn next(&mut self, type_id: i8) -> Option<Self::Enum>;
}

/// Types that may be consumed to construct iterators of
/// union array variants.
pub trait UnionArrayIterators {
    /// Type holding the variant iterators that may be
    /// advanced to get the value for a `type_id`.
    type VariantIterators: TypeIdIterator;

    /// Constructs `VariantIterators` by consuming `Self`.
    fn new_variant_iters(self) -> Self::VariantIterators;
}

/// Type holding the variant iterators of a union array
type VarIters<T, const VARIANTS: usize, Buffer, OffsetItem, UnionLayout> = <<T as UnionArrayType<
    VARIANTS,
>>::Array<
    Buffer,
    OffsetItem,
    UnionLayout,
> as UnionArrayIterators>::VariantIterators;

/// State required to iterate over union arrays
pub struct UnionArrayIntoIter<
    T: UnionArrayType<VARIANTS>,
    const VARIANTS: usize,
    Buffer: BufferType,
    OffsetItem: Offset,
    UnionLayout: UnionType,
> where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, UnionLayout>: UnionArrayIterators,
    <Buffer as BufferType>::Buffer<i8>: IntoIterator<Item = i8>,
{
    /// Type ids of the items in the union array
    type_ids: <Int8Array<NonNullable, Buffer> as IntoIterator>::IntoIter,

    /// Iterators of variants that may be advanced to get items for a type id
    variant_iterators: VarIters<T, VARIANTS, Buffer, OffsetItem, UnionLayout>,
}

impl<
    T: UnionArrayType<VARIANTS>,
    const VARIANTS: usize,
    Buffer: BufferType,
    OffsetItem: Offset,
    UnionLayout: UnionType,
> Iterator for UnionArrayIntoIter<T, VARIANTS, Buffer, OffsetItem, UnionLayout>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, UnionLayout>: UnionArrayIterators,
    <Buffer as BufferType>::Buffer<i8>: IntoIterator<Item = i8>,
    VarIters<T, VARIANTS, Buffer, OffsetItem, UnionLayout>: TypeIdIterator<Enum = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.type_ids.next().map(|type_id| {
            self.variant_iterators
                .next(type_id)
                .expect("child arrays have correct length")
        })
    }
}

impl<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: Offset>
    IntoIterator for UnionArray<T, VARIANTS, SparseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, SparseLayout>: UnionArrayIterators,
    <Buffer as BufferType>::Buffer<i8>: IntoIterator<Item = i8>,
    VarIters<T, VARIANTS, Buffer, OffsetItem, SparseLayout>: TypeIdIterator<Enum = T>,
{
    type Item = T;
    type IntoIter = UnionArrayIntoIter<T, VARIANTS, Buffer, OffsetItem, SparseLayout>;

    fn into_iter(self) -> Self::IntoIter {
        UnionArrayIntoIter {
            variant_iterators: UnionArrayIterators::new_variant_iters(self.0.variants),
            type_ids: self.0.types.into_iter(),
        }
    }
}

impl<T: UnionArrayType<VARIANTS>, const VARIANTS: usize, Buffer: BufferType, OffsetItem: Offset>
    IntoIterator for UnionArray<T, VARIANTS, DenseLayout, Buffer, OffsetItem>
where
    for<'a> i8: From<&'a T>,
    <T as UnionArrayType<VARIANTS>>::Array<Buffer, OffsetItem, DenseLayout>: UnionArrayIterators,
    <Buffer as BufferType>::Buffer<i8>: IntoIterator<Item = i8>,
    VarIters<T, VARIANTS, Buffer, OffsetItem, DenseLayout>: TypeIdIterator<Enum = T>,
{
    type Item = T;
    type IntoIter = UnionArrayIntoIter<T, VARIANTS, Buffer, OffsetItem, DenseLayout>;

    fn into_iter(self) -> Self::IntoIter {
        UnionArrayIntoIter {
            variant_iterators: UnionArrayIterators::new_variant_iters(self.0.variants),
            type_ids: self.0.types.into_iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::array::Uint32Array;
    use std::marker::PhantomData;

    #[test]
    #[rustversion::attr(nightly, allow(non_local_definitions))]
    #[allow(clippy::too_many_lines)]
    fn simple() {
        #[derive(Clone, Debug, PartialEq, Eq)]
        enum Foo {
            Bar(i32),
            Baz(u32),
        }

        struct FooArray<Buffer: BufferType, UnionLayout: UnionType> {
            bar: Int32Array<NonNullable, Buffer>,
            baz: Uint32Array<NonNullable, Buffer>,
            _ty: PhantomData<UnionLayout>, // we can also use a const generic instead?
        }

        impl<Buffer: BufferType> DenseOffset for FooArray<Buffer, DenseLayout> {
            fn variant_len(&self, type_id: i8) -> usize {
                match type_id {
                    0 => self.bar.len(),
                    1 => self.baz.len(),
                    _ => panic!("bad type id"),
                }
            }
        }

        impl<Buffer: BufferType, UnionLayout: UnionType> Clone for FooArray<Buffer, UnionLayout>
        where
            Int32Array<NonNullable, Buffer>: Clone,
            Uint32Array<NonNullable, Buffer>: Clone,
        {
            fn clone(&self) -> Self {
                Self {
                    bar: self.bar.clone(),
                    baz: self.baz.clone(),
                    _ty: self._ty,
                }
            }
        }

        impl<Buffer: BufferType, UnionLayout: UnionType> Default for FooArray<Buffer, UnionLayout>
        where
            Int32Array<NonNullable, Buffer>: Default,
            Uint32Array<NonNullable, Buffer>: Default,
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
            Int32Array<NonNullable, Buffer>: Extend<i32>,
            Uint32Array<NonNullable, Buffer>: Extend<u32>,
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
            Int32Array<NonNullable, Buffer>: Extend<i32>,
            Uint32Array<NonNullable, Buffer>: Extend<u32>,
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

        struct FooArrayIntoIter<Buffer: BufferType, UnionLayout: UnionType>
        where
            Int32Array<NonNullable, Buffer>: IntoIterator,
            Uint32Array<NonNullable, Buffer>: IntoIterator,
        {
            bar: <Int32Array<NonNullable, Buffer> as IntoIterator>::IntoIter,
            baz: <Uint32Array<NonNullable, Buffer> as IntoIterator>::IntoIter,
            _ty: PhantomData<UnionLayout>,
        }

        impl<Buffer: BufferType> TypeIdIterator for FooArrayIntoIter<Buffer, DenseLayout>
        where
            Int32Array<NonNullable, Buffer>: IntoIterator<Item = i32>,
            Uint32Array<NonNullable, Buffer>: IntoIterator<Item = u32>,
        {
            type Enum = Foo;
            fn next(&mut self, type_id: i8) -> Option<Self::Enum> {
                match type_id {
                    0 => self.bar.next().map(Foo::Bar),
                    1 => self.baz.next().map(Foo::Baz),
                    _ => panic!("type id greater than number of variants"),
                }
            }
        }

        impl<Buffer: BufferType> TypeIdIterator for FooArrayIntoIter<Buffer, SparseLayout>
        where
            Int32Array<NonNullable, Buffer>: IntoIterator<Item = i32>,
            Uint32Array<NonNullable, Buffer>: IntoIterator<Item = u32>,
        {
            type Enum = Foo;
            fn next(&mut self, type_id: i8) -> Option<Self::Enum> {
                match type_id {
                    0 => {
                        self.baz.next();
                        self.bar.next().map(Foo::Bar)
                    }
                    1 => {
                        self.bar.next();
                        self.baz.next().map(Foo::Baz)
                    }
                    _ => panic!("type id greater than number of variants"),
                }
            }
        }

        impl<Buffer: BufferType, UnionLayout: UnionType> UnionArrayIterators
            for FooArray<Buffer, UnionLayout>
        where
            Int32Array<NonNullable, Buffer>: IntoIterator<Item = i32>,
            Uint32Array<NonNullable, Buffer>: IntoIterator<Item = u32>,
            FooArrayIntoIter<Buffer, UnionLayout>: TypeIdIterator,
        {
            type VariantIterators = FooArrayIntoIter<Buffer, UnionLayout>;

            fn new_variant_iters(self) -> Self::VariantIterators {
                Self::VariantIterators {
                    bar: self.bar.into_iter(),
                    baz: self.baz.into_iter(),
                    _ty: PhantomData,
                }
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
            type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
                FooArray<Buffer, UnionLayout>;
        }

        {
            let input = vec![Foo::Bar(0), Foo::Baz(1), Foo::Baz(2), Foo::Bar(3)];
            let mut dense_array = input
                .clone()
                .into_iter()
                .collect::<UnionArray<Foo, { Foo::VARIANTS }>>();

            assert_eq!(dense_array.0.types.0, [0, 1, 1, 0]);
            assert_eq!(dense_array.0.offsets.0, [0, 0, 1, 1]);
            assert_eq!(dense_array.0.variants.bar.0, [0, 3]);
            assert_eq!(dense_array.0.variants.baz.0, [1, 2]);

            assert_eq!(dense_array.clone().into_iter().collect::<Vec<_>>(), input);

            dense_array.extend(iter::once(Foo::Bar(42)));
            assert_eq!(dense_array.0.types.0, [0, 1, 1, 0, 0]);
            assert_eq!(dense_array.0.offsets.0, [0, 0, 1, 1, 2]);
            assert_eq!(dense_array.0.variants.bar.0, [0, 3, 42]);
            assert_eq!(dense_array.0.variants.baz.0, [1, 2]);
        };

        {
            let input = vec![Foo::Bar(-78), Foo::Baz(1), Foo::Baz(99)];
            let mut sparse_array = input.clone().into_iter().collect::<UnionArray<
                Foo,
                { Foo::VARIANTS },
                SparseLayout,
            >>();

            assert_eq!(sparse_array.0.types.0, [0, 1, 1]);
            assert_eq!(
                sparse_array.0.variants.bar.0,
                [-78, i32::default(), i32::default()]
            );
            assert_eq!(sparse_array.0.variants.baz.0, [u32::default(), 1, 99]);

            assert_eq!(sparse_array.clone().into_iter().collect::<Vec<_>>(), input);

            sparse_array.extend(iter::once(Foo::Bar(42)));
            assert_eq!(sparse_array.0.types.0, [0, 1, 1, 0]);
            assert_eq!(
                sparse_array.0.variants.bar.0,
                [-78, i32::default(), i32::default(), 42]
            );
            assert_eq!(
                sparse_array.0.variants.baz.0,
                [u32::default(), 1, 99, u32::default()]
            );
        };
    }

    #[test]
    #[cfg(feature = "derive")]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::type_complexity)]
    #[allow(unused)]
    #[rustversion::attr(nightly, allow(non_local_definitions))]
    fn with_multiple_fields() {
        use crate::{ArrayType, Length, array::ArrayTypeOf, offset};

        #[derive(Clone, Debug, PartialEq, Eq)]
        enum Foo {
            Unit,
            Unnamed(u8, u16),
            Named { a: u32, b: u64, c: String },
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
            c: String,
        }

        impl EnumVariant<2> for Foo {
            type Data = FooVariantNamed;

            fn from_data(value: Self::Data) -> Self {
                Self::Named {
                    a: value.a,
                    b: value.b,
                    c: value.c,
                }
            }
        }

        struct FooArray<Buffer: BufferType, UnionLayout: UnionType> {
            unit: ArrayTypeOf<<Foo as EnumVariant<0>>::Data, Buffer, offset::NA, UnionLayout>,
            unnamed: ArrayTypeOf<<Foo as EnumVariant<1>>::Data, Buffer, offset::NA, UnionLayout>,
            named: ArrayTypeOf<<Foo as EnumVariant<2>>::Data, Buffer, offset::NA, UnionLayout>,
        }

        impl<Buffer: BufferType, UnionLayout: UnionType> Default for FooArray<Buffer, UnionLayout>
        where
            ArrayTypeOf<<Foo as EnumVariant<0>>::Data, Buffer, offset::NA, UnionLayout>: Default,
            ArrayTypeOf<<Foo as EnumVariant<1>>::Data, Buffer, offset::NA, UnionLayout>: Default,
            ArrayTypeOf<<Foo as EnumVariant<2>>::Data, Buffer, offset::NA, UnionLayout>: Default,
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
            ArrayTypeOf<<Foo as EnumVariant<0>>::Data, Buffer, offset::NA, DenseLayout>:
                Extend<<Foo as EnumVariant<0>>::Data>,
            ArrayTypeOf<<Foo as EnumVariant<1>>::Data, Buffer, offset::NA, DenseLayout>:
                Extend<<Foo as EnumVariant<1>>::Data>,
            ArrayTypeOf<<Foo as EnumVariant<2>>::Data, Buffer, offset::NA, DenseLayout>:
                Extend<<Foo as EnumVariant<2>>::Data>,
        {
            fn extend<T: IntoIterator<Item = Foo>>(&mut self, iter: T) {
                iter.into_iter().for_each(|item| match item {
                    Foo::Unit => self.unit.extend(iter::once(())),
                    Foo::Unnamed(a, b) => self.unnamed.extend(iter::once(FooVariantUnnamed(a, b))),
                    Foo::Named { a, b, c } => {
                        self.named.extend(iter::once(FooVariantNamed { a, b, c }));
                    }
                });
            }
        }

        impl<Buffer: BufferType> Extend<Foo> for FooArray<Buffer, SparseLayout>
        where
            ArrayTypeOf<<Foo as EnumVariant<0>>::Data, Buffer, offset::NA, SparseLayout>:
                Extend<<Foo as EnumVariant<0>>::Data>,
            ArrayTypeOf<<Foo as EnumVariant<1>>::Data, Buffer, offset::NA, SparseLayout>:
                Extend<<Foo as EnumVariant<1>>::Data>,
            ArrayTypeOf<<Foo as EnumVariant<2>>::Data, Buffer, offset::NA, SparseLayout>:
                Extend<<Foo as EnumVariant<2>>::Data>,
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
                    Foo::Named { a, b, c } => {
                        self.unit.extend(iter::once(()));
                        self.unnamed
                            .extend(iter::once(FooVariantUnnamed::default()));
                        self.named.extend(iter::once(FooVariantNamed { a, b, c }));
                    }
                });
            }
        }

        type FooEnumVariantArray<const INDEX: usize, Buffer, UnionLayout> =
            ArrayTypeOf<<Foo as EnumVariant<INDEX>>::Data, Buffer, offset::NA, UnionLayout>;

        struct FooArrayIntoIter<Buffer: BufferType, UnionLayout: UnionType>
        where
            FooEnumVariantArray<0, Buffer, UnionLayout>: IntoIterator,
            FooEnumVariantArray<1, Buffer, UnionLayout>: IntoIterator,
            FooEnumVariantArray<2, Buffer, UnionLayout>: IntoIterator,
        {
            unit: <FooEnumVariantArray<0, Buffer, UnionLayout> as IntoIterator>::IntoIter,
            unnamed: <FooEnumVariantArray<1, Buffer, UnionLayout> as IntoIterator>::IntoIter,
            named: <FooEnumVariantArray<2, Buffer, UnionLayout> as IntoIterator>::IntoIter,
        }

        impl<Buffer: BufferType> TypeIdIterator for FooArrayIntoIter<Buffer, DenseLayout>
        where
            FooEnumVariantArray<0, Buffer, DenseLayout>:
                IntoIterator<Item = <Foo as EnumVariant<0>>::Data>,
            FooEnumVariantArray<1, Buffer, DenseLayout>:
                IntoIterator<Item = <Foo as EnumVariant<1>>::Data>,
            FooEnumVariantArray<2, Buffer, DenseLayout>:
                IntoIterator<Item = <Foo as EnumVariant<2>>::Data>,
        {
            type Enum = Foo;

            fn next(&mut self, type_id: i8) -> Option<Self::Enum> {
                match type_id {
                    0 => self.unit.next().map(<Foo as EnumVariant<0>>::from_data),
                    1 => self.unnamed.next().map(<Foo as EnumVariant<1>>::from_data),
                    2 => self.named.next().map(<Foo as EnumVariant<2>>::from_data),
                    _ => panic!("type id greater than number of variants"),
                }
            }
        }

        impl<Buffer: BufferType> TypeIdIterator for FooArrayIntoIter<Buffer, SparseLayout>
        where
            FooEnumVariantArray<0, Buffer, SparseLayout>:
                IntoIterator<Item = <Foo as EnumVariant<0>>::Data>,
            FooEnumVariantArray<1, Buffer, SparseLayout>:
                IntoIterator<Item = <Foo as EnumVariant<1>>::Data>,
            FooEnumVariantArray<2, Buffer, SparseLayout>:
                IntoIterator<Item = <Foo as EnumVariant<2>>::Data>,
        {
            type Enum = Foo;

            fn next(&mut self, type_id: i8) -> Option<Self::Enum> {
                match type_id {
                    0 => {
                        let to_return = self.unit.next().map(<Foo as EnumVariant<0>>::from_data);
                        self.unnamed.next();
                        self.named.next();

                        to_return
                    }
                    1 => {
                        self.unit.next();
                        let to_return = self.unnamed.next().map(<Foo as EnumVariant<1>>::from_data);
                        self.named.next();

                        to_return
                    }
                    2 => {
                        self.unit.next();
                        self.unnamed.next();
                        let to_return = self.named.next().map(<Foo as EnumVariant<2>>::from_data);

                        #[allow(clippy::let_and_return)]
                        to_return
                    }
                    _ => panic!("type id greater than number of variants"),
                }
            }
        }

        impl<Buffer: BufferType, UnionLayout: UnionType> UnionArrayIterators
            for FooArray<Buffer, UnionLayout>
        where
            FooEnumVariantArray<0, Buffer, SparseLayout>:
                IntoIterator<Item = <Foo as EnumVariant<0>>::Data>,
            FooEnumVariantArray<1, Buffer, SparseLayout>:
                IntoIterator<Item = <Foo as EnumVariant<1>>::Data>,
            FooEnumVariantArray<2, Buffer, SparseLayout>:
                IntoIterator<Item = <Foo as EnumVariant<2>>::Data>,
            FooArrayIntoIter<Buffer, UnionLayout>: TypeIdIterator,
        {
            type VariantIterators = FooArrayIntoIter<Buffer, UnionLayout>;

            fn new_variant_iters(self) -> Self::VariantIterators {
                Self::VariantIterators {
                    unit: self.unit.into_iter(),
                    unnamed: self.unnamed.into_iter(),
                    named: self.named.into_iter(),
                }
            }
        }

        impl UnionArrayType<3> for Foo {
            type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
                FooArray<Buffer, UnionLayout>;
        }

        impl ArrayType<Foo> for Foo {
            type Array<Buffer: BufferType, OffsetItem: offset::Offset, UnionLayout: UnionType> =
                UnionArray<Foo, { Foo::VARIANTS }, UnionLayout>;
        }

        {
            let input = vec![
                Foo::Unit,
                Foo::Unnamed(1, 2),
                Foo::Named {
                    a: 3,
                    b: 4,
                    c: "woo hoo!".to_owned(),
                },
            ];
            let dense_array = input
                .clone()
                .into_iter()
                .collect::<UnionArray<Foo, { Foo::VARIANTS }>>();

            assert_eq!(dense_array.0.types.0, [0, 1, 2]);
            assert_eq!(dense_array.0.offsets.0, [0, 0, 0]);
            assert_eq!(dense_array.0.variants.unit.0.len(), 1);
            assert_eq!(dense_array.0.variants.unnamed.0.0.0, [1]);
            assert_eq!(dense_array.0.variants.unnamed.0.1.0, [2]);
            assert_eq!(dense_array.0.variants.named.0.a.0, [3]);
            assert_eq!(dense_array.0.variants.named.0.b.0, [4]);

            assert_eq!(dense_array.into_iter().collect::<Vec<_>>(), input);
        };

        {
            let input = vec![
                Foo::Unit,
                Foo::Unnamed(1, 2),
                Foo::Named {
                    a: 3,
                    b: 4,
                    c: "woo hoo!".to_owned(),
                },
            ];
            let sparse_array = input.clone().into_iter().collect::<UnionArray<
                Foo,
                { Foo::VARIANTS },
                SparseLayout,
            >>();
            assert_eq!(sparse_array.into_iter().collect::<Vec<_>>(), input);
        };
    }

    #[test]
    #[cfg(feature = "derive")]
    #[rustversion::attr(nightly, allow(non_local_definitions))]
    fn derive() {
        use crate::ArrayType;

        #[derive(ArrayType, Copy, Clone, Default, Debug, PartialEq, Eq)]
        enum Foo {
            #[default]
            Foo,
            Bar,
        }

        #[derive(ArrayType, Clone, Copy, Debug, PartialEq, Eq)]
        enum Test {
            Foo { bar: u8 },
            Bar(bool),
            None,
        }

        let foo_input = vec![Foo::Foo, Foo::Bar];

        let dense_foo_array = foo_input.clone().into_iter().collect::<UnionArray<
            Foo,
            { Foo::VARIANTS },
            DenseLayout,
        >>();
        assert_eq!(dense_foo_array.len(), foo_input.len());
        let a = dense_foo_array.into_iter().collect::<Vec<_>>();
        assert_eq!(a, foo_input.clone());

        let sparse_foo_array = foo_input.clone().into_iter().collect::<UnionArray<
            Foo,
            { <Foo as UnionArrayType<2>>::VARIANTS },
            SparseLayout,
        >>();
        assert_eq!(sparse_foo_array.len(), foo_input.len());
        assert_eq!(sparse_foo_array.into_iter().collect::<Vec<_>>(), foo_input);

        let input = vec![
            Test::None,
            Test::Bar(true),
            Test::Foo { bar: 123 },
            Test::None,
        ];
        let mut dense_array = input
            .clone()
            .into_iter()
            .collect::<UnionArray<Test, { <Test as UnionArrayType<3>>::VARIANTS }>>();
        assert_eq!(dense_array.len(), 4);
        assert_eq!(dense_array.0.types.0, &[2, 1, 0, 2]);
        assert_eq!(dense_array.0.offsets.0, &[0, 0, 0, 1]);
        assert_eq!(dense_array.0.variants.0.0.bar.0, &[123]);
        assert_eq!(dense_array.0.variants.2.0.len(), 2);
        assert_eq!(
            dense_array.clone().into_iter().collect::<Vec<_>>(),
            input.clone()
        );

        dense_array.extend(iter::once(Test::Foo { bar: 42 }));
        assert_eq!(dense_array.len(), 5);
        assert_eq!(dense_array.0.types.0, &[2, 1, 0, 2, 0]);
        assert_eq!(dense_array.0.offsets.0, &[0, 0, 0, 1, 1]);
        assert_eq!(dense_array.0.variants.0.0.bar.0, &[123, 42]);
        assert_eq!(dense_array.0.variants.2.0.len(), 2);

        let sparse_array = input.clone().into_iter().collect::<UnionArray<
            Test,
            { <Test as UnionArrayType<3>>::VARIANTS },
            SparseLayout,
        >>();
        assert_eq!(sparse_array.len(), 4);
        assert_eq!(sparse_array.0.types.0, &[2, 1, 0, 2]);
        assert_eq!(sparse_array.0.variants.0.0.bar.0, &[0, 0, 123, 0]);
        assert_eq!(sparse_array.0.variants.2.0.len(), 4);
        assert_eq!(sparse_array.into_iter().collect::<Vec<_>>(), input);
    }
}
