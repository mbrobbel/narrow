use crate::{Array, ArrayIndex, ArrayType};
use std::{
    iter::{self, Repeat, Take},
    marker::PhantomData,
};

/// A sequence of nulls.
///
/// This array type is also used as [ArrayType] when deriving [Array] for types
/// without fields (unit types). The generic `T` is used to provide iterator
/// implementations for array of these unit types.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NullArray<T = ()> {
    len: usize,
    _ty: PhantomData<fn() -> T>,
}

impl<T> NullArray<T> {
    /// Returns a new NullArray with the given length.
    ///
    /// This never allocates.
    pub fn with_len(len: usize) -> Self {
        Self {
            len,
            _ty: PhantomData,
        }
    }

    /// Returns the number of elements in the array, also referred to as its
    /// length.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the array contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Array for NullArray {
    type Validity = Self;

    fn validity(&self) -> &Self::Validity {
        self
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn null_count(&self) -> usize {
        self.len
    }

    fn all_null(&self) -> bool {
        true
    }

    fn is_null(&self, index: usize) -> bool {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("is_null index (is {}) should be < len (is {})", index, len);
        }

        let len = self.len;
        if index >= len {
            assert_failed(index, len);
        }

        true
    }

    fn valid_count(&self) -> usize {
        0
    }

    fn any_valid(&self) -> bool {
        false
    }

    fn all_valid(&self) -> bool {
        false
    }

    fn is_valid(&self, index: usize) -> bool {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("is_valid index (is {}) should be < len (is {})", index, len);
        }

        let len = self.len;
        if index >= len {
            assert_failed(index, len);
        }

        false
    }
}

impl<T> ArrayIndex<usize> for NullArray<T>
where
    T: Default,
{
    type Output = T;

    fn index(&self, index: usize) -> Self::Output {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("index (is {}) should be < len (is {})", index, len);
        }

        let len = self.len;
        if index >= len {
            assert_failed(index, len);
        }

        T::default()
    }
}

impl ArrayType for () {
    type Array = NullArray;
}

impl<T> FromIterator<T> for NullArray<T> {
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
    T: Clone + Default,
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
        let array = vec.into_iter().collect::<NullArray>();
        assert_eq!(array.len(), 100);
        assert!(array.is_null(0));
    }

    #[test]
    fn into_iter() {
        let vec = vec![(); 100];
        let array = vec.iter().copied().collect::<NullArray>();
        assert_eq!(vec, array.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn unit_type() {
        #[derive(Clone, Default, Debug, PartialEq)]
        struct UnitStruct;

        let vec = vec![UnitStruct; 100];
        let array = vec.iter().cloned().collect::<NullArray<_>>();
        assert_eq!(array.len(), 100);
        assert_eq!(vec, array.into_iter().collect::<Vec<_>>());

        #[derive(Clone, Debug, PartialEq)]
        enum UnitEnum {
            Unit,
        }

        impl Default for UnitEnum {
            fn default() -> Self {
                UnitEnum::Unit
            }
        }

        let vec = vec![UnitEnum::default(); 100];
        let array = vec.iter().cloned().collect::<NullArray<_>>();
        assert_eq!(array.len(), 100);
        assert_eq!(vec, array.into_iter().collect::<Vec<_>>());
    }
}
