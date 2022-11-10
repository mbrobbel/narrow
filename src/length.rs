use std::{collections::VecDeque, rc::Rc, sync::Arc};

/// The length (or number of elements) of a collection.
pub trait Length {
    /// Returns the number of elements in the collection, also referred to as
    /// its length.
    fn len(&self) -> usize;

    /// Returns `true` if there are no elements in the collection.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T, const N: usize> Length for [T; N] {
    #[inline]
    fn len(&self) -> usize {
        N
    }
}

impl<T> Length for &[T] {
    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> Length for Vec<T> {
    #[inline]
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl<T> Length for Box<[T]> {
    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> Length for Rc<[T]> {
    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> Length for Arc<[T]> {
    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> Length for VecDeque<T> {
    #[inline]
    fn len(&self) -> usize {
        VecDeque::len(self)
    }
}

impl<T> Length for &T
where
    T: Length,
{
    #[inline]
    fn len(&self) -> usize {
        T::len(self)
    }
}

impl Length for &str {
    #[inline]
    fn len(&self) -> usize {
        str::len(self)
    }
}
