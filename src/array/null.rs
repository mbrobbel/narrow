use crate::{Array, ArrayIndex, ArrayType};
use std::iter::{self, FromIterator, Repeat, Take};

/// A sequence of nulls.
#[derive(Debug)]
pub struct NullArray {
    len: usize,
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

impl ArrayIndex<usize> for NullArray {
    type Output = ();

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
    }
}

impl ArrayType for () {
    type Array = NullArray;
}

impl FromIterator<()> for NullArray {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = ()>,
    {
        Self {
            len: iter.into_iter().count(),
        }
    }
}

impl<'a> IntoIterator for &'a NullArray {
    type Item = ();
    type IntoIter = Take<Repeat<()>>;

    fn into_iter(self) -> Self::IntoIter {
        iter::repeat(()).take(self.len)
    }
}
