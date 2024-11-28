//! Indexing operations.

use crate::Length;
use std::{collections::VecDeque, rc::Rc, sync::Arc};

/// Index operation for shared access to values in a collection.
pub trait Index: Length {
    /// The item.
    type Item<'a>
    where
        Self: 'a;

    /// Returns the value at given index. Returns `None` if the index is out of range.
    fn index(&self, index: usize) -> Option<Self::Item<'_>> {
        (index < self.len()).then(||
            // Safety:
            // - Bounds checked in predicate
            unsafe { self.index_unchecked(index)})
    }

    /// Returns the value at given index. Panics if the index is out of bounds.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    fn index_checked(&self, index: usize) -> Self::Item<'_> {
        /// Panic when out of bounds.
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("index (is {index}) should be < len (is {len})");
        }

        let len = self.len();
        if index >= len {
            assert_failed(index, len);
        }

        // Safety:
        // - Bounds checked above.
        unsafe { self.index_unchecked(index) }
    }

    /// Returns the value at given index. Skips bound checking.
    ///
    /// # Safety
    ///
    /// Caller must ensure index is within bounds.
    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_>;
}

impl<T> Index for Vec<T> {
    type Item<'a>
        = &'a T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.get_unchecked(index)
    }
}

impl<T, const N: usize> Index for [T; N] {
    type Item<'a>
        = &'a T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.get_unchecked(index)
    }
}

impl<T> Index for [T] {
    type Item<'a>
        = &'a T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.get_unchecked(index)
    }
}

impl<T> Index for &[T] {
    type Item<'a>
        = &'a T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.get_unchecked(index)
    }
}

impl<T> Index for &mut [T] {
    type Item<'a>
        = &'a T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.get_unchecked(index)
    }
}

impl<T> Index for Box<[T]> {
    type Item<'a>
        = &'a T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.get_unchecked(index)
    }
}

impl<T> Index for Rc<[T]> {
    type Item<'a>
        = &'a T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.get_unchecked(index)
    }
}

impl<T> Index for Arc<[T]> {
    type Item<'a>
        = &'a T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.get_unchecked(index)
    }
}

impl<T> Index for VecDeque<T> {
    type Item<'a>
        = &'a T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        // VecDeque has no unchecked get so this panics for out of bounds access
        std::ops::Index::index(self, index)
    }
}
