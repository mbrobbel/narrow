use crate::{Array, ArrayType, Length, Null};
use std::{
    iter::{self, Repeat, Take},
    marker::PhantomData,
};

/// A sequence of nulls.
///
/// This array type is also used as [ArrayType] when deriving [Array] for types
/// without fields (unit types). The generic `T` is used to provide iterator
/// implementations for arrays of these unit types.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NullArray<T = ()> {
    len: usize,
    // Covariant over `T`: always Send + Sync
    _ty: PhantomData<fn() -> T>,
}

/// A marker trait for unit types.
///
/// It is derived automatically for types without fields that have [NullArray]
/// as [ArrayType], and used as a trait bound on the methods that are used to
/// support deriving [Array] for these types.
///
/// This trait is unsafe because the compiler can't verify that it only gets
/// implemented by unit types.
///
/// The [Default] implementation must return the only allowed value of this unit
/// type.
pub unsafe trait Unit
where
    Self: Default,
{
}

impl<T> NullArray<T> {
    /// Constructs a new `NullArray<T>` with the specified length.
    ///
    /// This never allocates.
    pub fn new(len: usize) -> Self {
        Self {
            len,
            _ty: PhantomData,
        }
    }
}

impl<T> Array for NullArray<T>
where
    T: ArrayType + Unit,
{
    type Item<'a>
    where
        Self: 'a,
    = T;
}

// Safety:
// - std::mem::size_of::<()> == 0
unsafe impl Unit for () {}

impl ArrayType for () {
    // type Item<'a> = ();
    type RefItem<'a> = &'a ();
    type Array<T, const N: bool> = NullArray<()>;
}

impl<T> Length for NullArray<T> {
    fn len(&self) -> usize {
        self.len
    }
}

// todo(mb): check the compiler output for provided methods
impl<T> Null for NullArray<T> {
    unsafe fn is_valid_unchecked(&self, _index: usize) -> bool {
        // All elements are null in a NullArray.
        false
    }
}

impl<'a, T> FromIterator<&'a T> for NullArray<T>
where
    T: Unit,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a T>,
    {
        Self {
            len: iter.into_iter().count(),
            _ty: PhantomData,
        }
    }
}

impl<T> FromIterator<T> for NullArray<T>
where
    T: Unit,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            len: iter.into_iter().count(),
            _ty: PhantomData,
        }
    }
}

impl<'a, T> IntoIterator for &'a NullArray<T>
where
    T: Clone + Default + Unit,
{
    type Item = T;
    type IntoIter = Take<Repeat<T>>;

    fn into_iter(self) -> Self::IntoIter {
        iter::repeat(T::default()).take(self.len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        let vec = vec![(); 100];
        let array = vec.iter().collect::<NullArray>();
        assert_eq!(array.len(), 100);
        assert_eq!(array.is_null(0), Some(true));
    }

    #[test]
    fn into_iter() {
        let vec = vec![(); 100];
        let array = vec.iter().collect::<NullArray>();
        assert_eq!(vec, array.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn unit_type() {
        #[derive(Clone, Default, Debug, PartialEq)]
        struct UnitStruct;
        assert_eq!(std::mem::size_of::<UnitStruct>(), 0);
        unsafe impl Unit for UnitStruct {}

        let vec = vec![UnitStruct; 100];
        let array = vec.iter().collect::<NullArray<_>>();
        assert_eq!(array.len(), 100);
        assert_eq!(vec, array.into_iter().collect::<Vec<_>>());

        #[derive(Clone, Debug, PartialEq)]
        enum UnitEnum {
            Unit,
        }
        assert_eq!(std::mem::size_of::<UnitEnum>(), 0);
        unsafe impl Unit for UnitEnum {}
        impl Default for UnitEnum {
            fn default() -> Self {
                UnitEnum::Unit
            }
        }

        let vec = vec![UnitEnum::default(); 100];
        let array = vec.iter().collect::<NullArray<_>>();
        assert_eq!(array.len(), 100);
        assert_eq!(vec, array.into_iter().collect::<Vec<_>>());
    }
}
