//! Length of items.

extern crate alloc;

use alloc::{boxed::Box, collections::VecDeque, rc::Rc, sync::Arc, vec::Vec};

/// The length (or number of units) of an item.
pub trait Length {
    /// Returns the number of units in this item, also referred to as its length.
    fn len(&self) -> usize;

    /// Returns `true` if there are no unit in this item.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Length for Vec<T> {
    fn len(&self) -> usize {
        Self::len(self)
    }
}

impl<T, const N: usize> Length for [T; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<T> Length for [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> Length for &[T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> Length for &mut [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> Length for Box<[T]> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

impl<T> Length for Rc<[T]> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

impl<T> Length for Arc<[T]> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

impl<T> Length for VecDeque<T> {
    fn len(&self) -> usize {
        Self::len(self)
    }
}
