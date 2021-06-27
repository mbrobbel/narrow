use crate::{Array, ArrayData, ArrayType, Nullable, Validity};
use std::{fmt::Debug, iter::FromIterator, ops::Deref};

// todo(mb): add partial eq with array of struct

/// Struct types that can be stored as an array.
///
/// Enables converting arrays of structs into structs of arrays.
pub trait StructArrayType
where
    Self: Sized,
{
    /// The type storing the struct of arrays.
    // ArrayData because the struct array does not contain the validity of the
    // StructArray.
    type Array: FromIterator<Self> + ArrayData;
}

// todo(mb): remove when GATs
// or(add generic T to ArrayType to allow impl ArrayType<T> for Option<T> in derive macro)
impl<T> ArrayType for Option<T>
where
    T: StructArrayType,
{
    type Array = StructArray<T, true>;
}

/// Array with structs that have fields of other array types.
pub struct StructArray<T, const N: bool>(Validity<<T as StructArrayType>::Array, N>)
where
    T: StructArrayType;

impl<T, const N: bool> Debug for StructArray<T, N>
where
    T: StructArrayType,
    Validity<<T as StructArrayType>::Array, N>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StructArray").field(&self.0).finish()
    }
}

impl<T, const N: bool> Array for StructArray<T, N>
where
    T: StructArrayType,
{
    type Validity = Validity<<T as StructArrayType>::Array, N>;

    fn validity(&self) -> &Self::Validity {
        &self.0
    }
}

impl<T, const N: bool> Deref for StructArray<T, N>
where
    T: StructArrayType,
{
    type Target = Validity<<T as StructArrayType>::Array, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromIterator<T> for StructArray<T, false>
where
    T: StructArrayType,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<T> FromIterator<Option<T>> for StructArray<T, true>
where
    T: StructArrayType + Default,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<T>>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<'a, T> IntoIterator for &'a StructArray<T, false>
where
    T: StructArrayType,
    // need GATs to add this to StructArrayType
    &'a <T as StructArrayType>::Array: IntoIterator<Item = T>,
{
    type Item = T;
    type IntoIter = <&'a <T as StructArrayType>::Array as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a StructArray<T, true>
where
    T: StructArrayType,
    &'a Nullable<<T as StructArrayType>::Array>: IntoIterator<Item = Option<T>>,
{
    type Item = Option<T>;
    type IntoIter = <&'a Nullable<<T as StructArrayType>::Array> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
