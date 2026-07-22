//! Length of items.

extern crate alloc;

use alloc::{boxed::Box, collections::VecDeque, rc::Rc, sync::Arc, vec::Vec};

/// The length (or number of units) of an item.
///
/// # Design
///
/// Arrow layouts are assembled from several physical collections that must
/// agree on their logical length. This small trait lets those collections
/// participate without requiring the full [`Collection`](crate::collection::Collection)
/// interface.
///
/// # Examples
///
/// ```
/// use narrow::length::Length;
///
/// assert_eq!(Length::len(&[1, 2, 3]), 3);
/// ```
pub trait Length {
    /// Returns the number of units in this item, also referred to as its length.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::length::Length;
    ///
    /// assert_eq!(Length::len(&[1, 2]), 2);
    /// ```
    fn len(&self) -> usize;

    /// Returns `true` if there are no unit in this item.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::length::Length;
    ///
    /// assert!(Length::is_empty(&[] as &[u8]));
    /// ```
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
