//! The length (number of elements) of a collection.

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

impl<const N: usize, T> Length for [T; N] {
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

impl<T> Length for &mut [T] {
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

impl Length for &str {
    fn len(&self) -> usize {
        str::len(self)
    }
}

impl Length for String {
    fn len(&self) -> usize {
        String::len(self)
    }
}

impl<T: Length> Length for Option<T> {
    fn len(&self) -> usize {
        match self {
            Some(item) => item.len(),
            None => 0,
        }
    }
}
