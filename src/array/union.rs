use crate::{Array, ArrayData, Int32Array, Int8Array, NestedArray};
use std::{
    fmt::Debug,
    iter::{Copied, Enumerate, Zip},
    ops::Deref,
    slice,
};

// todo(mb): sort fields (child arrays) based on size, so that nulls (types i8::default() and default value of first field are small).
// todo(mb): store variant arrays in array with sum type for all array types, then impl index<i8> for union array wrapper type to get arrays.
// todo(mb): impl UnionArrayType for std::Result and std::Option

/// The number of variants or types of union arrays.
pub trait UnionArrayVariants {
    /// The number of variants in this union.
    const VARIANTS: usize;
}

/// Union types that can be stored in arrays.
///
/// `D` encodes the union array type.
/// - [DenseUnionArray] when `D` is [true].
/// - [SparseUnionArray] when `D` is [false].
// todo(mb): GATs
pub trait UnionArrayType<const D: bool>: UnionArrayVariants + Sized {
    type Array: ArrayData;
    type Child: Array + UnionArrayIndex<Self> + FromIterator<Self>;
}

/// Array with an ordered sequence of variants of `T`.
/// `D` encodes the union array type.
/// - [DenseUnionArray] when `D` is [true].
/// - [SparseUnionArray] when `D` is [false].
pub struct UnionArray<T, const D: bool>(<T as UnionArrayType<D>>::Array)
where
    T: UnionArrayType<D>;

impl<T, const D: bool> Array for UnionArray<T, D>
where
    T: UnionArrayType<D>,
{
    type Validity = <T as UnionArrayType<D>>::Array;

    fn validity(&self) -> &Self::Validity {
        &self.0
    }
}

impl<T, const D: bool> Debug for UnionArray<T, D>
where
    T: UnionArrayType<D>,
    <T as UnionArrayType<D>>::Array: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("UnionArray").field(&self.0).finish()
    }
}

impl<T, const D: bool> Deref for UnionArray<T, D>
where
    T: UnionArrayType<D>,
{
    type Target = <T as UnionArrayType<D>>::Array;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const D: bool> FromIterator<T> for UnionArray<T, D>
where
    T: UnionArrayType<D>,
    <T as UnionArrayType<D>>::Array: FromIterator<T>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self(iter.into_iter().collect())
    }
}

// impl<'a, T, const D: bool> IntoIterator for &'a UnionArray<T, D>
// where
//     T: UnionArrayType<D>,
//     &'a <T as UnionArrayType<D>>::Array: IntoIterator<Item = T>,
// {
//     type Item = T;
//     type IntoIter = <&'a <T as UnionArrayType<D>>::Array as IntoIterator>::IntoIter;

//     fn into_iter(self) -> Self::IntoIter {
//         self.0.into_iter()
//     }
// }

/// Index trait for union arrays.
pub trait UnionArrayIndex<T> {
    fn index(&self, type_id: i8, index: i32) -> T;
}

/// Sparse union array for enum type `T`.
pub struct SparseUnionArray<T>
where
    T: UnionArrayType<false>,
    i8: for<'a> From<&'a T>,
{
    child: <T as UnionArrayType<false>>::Child,
    types: Int8Array<false>,
}

impl<T> Array for SparseUnionArray<T>
where
    T: UnionArrayType<false>,
    i8: for<'a> From<&'a T>,
{
    type Validity = Self;

    fn validity(&self) -> &Self::Validity {
        self
    }

    fn len(&self) -> usize {
        Array::len(&self.types)
    }

    fn null_count(&self) -> usize {
        // Nulls must be encoded by a variant.
        0
    }

    fn is_null(&self, _index: usize) -> bool {
        // todo(mb): bounds
        false
    }

    fn valid_count(&self) -> usize {
        Array::len(&self.types)
    }

    fn is_valid(&self, _index: usize) -> bool {
        // todo(mb): bounds
        true
    }
}

impl<T> NestedArray for SparseUnionArray<T>
where
    T: UnionArrayType<false>,
    i8: for<'a> From<&'a T>,
{
    type Child = <T as UnionArrayType<false>>::Child;

    fn child(&self) -> &Self::Child {
        &self.child
    }
}

impl<T> Debug for SparseUnionArray<T>
where
    T: UnionArrayType<false>,
    i8: for<'a> From<&'a T>,
    <T as UnionArrayType<false>>::Child: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SparseUnionArray")
            .field("child", &self.child)
            .field("types", &self.types)
            .finish()
    }
}

impl<T> FromIterator<T> for SparseUnionArray<T>
where
    T: UnionArrayType<false>,
    i8: for<'a> From<&'a T>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();
        let capacity = upper_bound.unwrap_or(lower_bound);

        let mut types = Vec::with_capacity(capacity);

        let child = iter
            .inspect(|item| {
                let type_id: i8 = item.into();
                types.push(type_id);
            })
            .collect();

        Self {
            child,
            types: types.into_iter().collect(),
        }
    }
}

impl<'a, T> IntoIterator for &'a SparseUnionArray<T>
where
    T: UnionArrayType<false>,
    i8: for<'b> From<&'b T>,
{
    type Item = T;
    type IntoIter = SparseUnionIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        SparseUnionIter {
            iter: self.types.into_iter().enumerate(),
            child: &self.child,
        }
    }
}

/// Iterator over elements of a sparse union array.
pub struct SparseUnionIter<'a, T>
where
    T: UnionArrayType<false>,
    i8: for<'b> From<&'b T>,
{
    iter: Enumerate<Copied<slice::Iter<'a, i8>>>,
    child: &'a <T as UnionArrayType<false>>::Child,
}

impl<'a, T> Iterator for SparseUnionIter<'a, T>
where
    T: UnionArrayType<false>,
    i8: for<'b> From<&'b T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(index, type_id)| self.child.index(type_id, index as i32))
    }
}

/// Dense union array for enum type `T` with `N` variants.
// todo(mb): remove const N (https://github.com/rust-lang/rust/issues/43408)
//           when we can use <T as UnionArrayType<true>>::VARIANTS in from_iter
//           `constant expression depends on a generic parameter`
pub struct DenseUnionArray<T, const N: usize>
where
    T: UnionArrayType<true>,
    i8: for<'a> From<&'a T>,
{
    child: <T as UnionArrayType<true>>::Child,
    types: Int8Array<false>,
    offsets: Int32Array<false>,
}

impl<T, const N: usize> Array for DenseUnionArray<T, N>
where
    T: UnionArrayType<true>,
    i8: for<'a> From<&'a T>,
{
    type Validity = Self;

    fn validity(&self) -> &Self::Validity {
        self
    }

    fn len(&self) -> usize {
        Array::len(&self.types)
    }

    fn null_count(&self) -> usize {
        // Nulls must be encoded by a variant.
        0
    }

    fn is_null(&self, _index: usize) -> bool {
        // todo(mb): bounds
        false
    }

    fn valid_count(&self) -> usize {
        Array::len(&self.types)
    }

    fn is_valid(&self, _index: usize) -> bool {
        // todo(mb): bounds
        true
    }
}

impl<T, const N: usize> NestedArray for DenseUnionArray<T, N>
where
    T: UnionArrayType<true>,
    i8: for<'a> From<&'a T>,
{
    type Child = <T as UnionArrayType<true>>::Child;

    fn child(&self) -> &Self::Child {
        &self.child
    }
}

impl<T, const N: usize> Debug for DenseUnionArray<T, N>
where
    T: UnionArrayType<true>,
    i8: for<'a> From<&'a T>,
    <T as UnionArrayType<true>>::Child: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DenseUnionArray")
            .field("child", &self.child)
            .field("types", &self.types)
            .field("offsets", &self.offsets)
            .finish()
    }
}

impl<T, const N: usize> FromIterator<T> for DenseUnionArray<T, N>
where
    T: UnionArrayType<true>,
    i8: for<'a> From<&'a T>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();
        let capacity = upper_bound.unwrap_or(lower_bound);

        let mut lens = [0; N];
        let mut types = Vec::with_capacity(capacity);
        let mut offsets = Vec::with_capacity(capacity);

        let child = iter
            .inspect(|item| {
                let type_id: i8 = item.into();
                types.push(type_id);
                let index = type_id as usize;
                offsets.push(lens[index]);
                lens[index] += 1;
            })
            .collect();

        Self {
            child,
            types: types.into_iter().collect(),
            offsets: offsets.into_iter().collect(),
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a DenseUnionArray<T, N>
where
    T: UnionArrayType<true>,
    i8: for<'b> From<&'b T>,
{
    type Item = T;
    type IntoIter = DenseUnionIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        DenseUnionIter {
            iter: self.types.into_iter().zip(self.offsets.into_iter()),
            child: &self.child,
        }
    }
}

/// Iterator over elements of a dense union array.
pub struct DenseUnionIter<'a, T>
where
    T: UnionArrayType<true>,
    i8: for<'b> From<&'b T>,
{
    iter: Zip<Copied<slice::Iter<'a, i8>>, Copied<slice::Iter<'a, i32>>>,
    child: &'a <T as UnionArrayType<true>>::Child,
}

impl<'a, T> Iterator for DenseUnionIter<'a, T>
where
    T: UnionArrayType<true>,
    i8: for<'b> From<&'b T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(type_id, offset)| self.child.index(type_id, offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_variant_field_less() {
        #[derive(Array, Copy, Clone, Debug, PartialEq)]
        enum Foo {
            Bar,
        }

        let vec = vec![Foo::Bar; 10];
        let sparse_array = vec.iter().copied().collect::<UnionArray<Foo, false>>();
        assert_eq!(sparse_array.into_iter().collect::<Vec<_>>(), vec);
        let dense_array = vec.iter().copied().collect::<UnionArray<Foo, true>>();
        assert_eq!(dense_array.into_iter().collect::<Vec<_>>(), vec);
    }

    #[test]
    fn one_variant_with_fields() {
        #[derive(Array, Copy, Clone, Debug, PartialEq)]
        enum Foo<'a, T> {
            Bar(&'a T),
        }

        let vec = vec![Foo::Bar(&1); 10];
        // let sparse_array = vec.iter().copied().collect::<UnionArray<Foo, false>>();
        // assert_eq!(sparse_array.into_iter().collect::<Vec<_>>(), vec);
        // let dense_array = vec.iter().copied().collect::<UnionArray<Foo, true>>();
        // assert_eq!(dense_array.into_iter().collect::<Vec<_>>(), vec);
    }

    // #[test]
    // fn from_iter() {
    //     #[derive(Array, Debug, Clone, PartialEq)]
    //     enum Union {
    //         Int(i8),
    //         Uint(u16),
    //         Bool(bool),
    //         OptBool(Option<bool>),
    //         List(Vec<u8>),
    //         String(String),
    //         None(()),
    //     }

    //     let vec = vec![
    //         Union::Int(1),
    //         Union::Int(2),
    //         Union::Int(3),
    //         Union::Int(4),
    //         Union::Uint(2),
    //         Union::Bool(false),
    //         Union::OptBool(Some(true)),
    //         Union::OptBool(None),
    //         Union::List(vec![1, 2, 3, 4]),
    //         Union::String("Hello world!".to_string()),
    //         Union::None(()),
    //     ];

    //     let dense_array = vec.clone().into_iter().collect::<UnionArray<Union, true>>();
    //     assert_eq!(Array::len(&dense_array.child().Int), 4);
    //     assert_eq!(Array::len(&dense_array.child().Uint), 1);
    //     assert_eq!(Array::len(&dense_array.child().Bool), 1);
    //     assert_eq!(Array::len(&dense_array.child().OptBool), 2);
    //     assert_eq!(Array::len(&dense_array.child().List), 1);
    //     assert_eq!(Array::len(&dense_array.child().String), 1);
    //     assert_eq!(Array::len(&dense_array.child().None), 1);
    //     assert_eq!(&dense_array.into_iter().collect::<Vec<_>>(), &vec);

    //     let sparse_array = vec
    //         .clone()
    //         .into_iter()
    //         .collect::<UnionArray<Union, false>>();
    //     assert_eq!(Array::len(&sparse_array.child().Int), 11);
    //     assert_eq!(Array::len(&sparse_array.child().Uint), 11);
    //     assert_eq!(Array::len(&sparse_array.child().Bool), 11);
    //     assert_eq!(Array::len(&sparse_array.child().OptBool), 11);
    //     assert_eq!(Array::len(&sparse_array.child().List), 11);
    //     assert_eq!(Array::len(&sparse_array.child().String), 11);
    //     assert_eq!(Array::len(&sparse_array.child().None), 11);
    //     assert_eq!(sparse_array.into_iter().collect::<Vec<_>>(), vec);
    // }
}
