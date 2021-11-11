use crate::Length;
use std::ops::Not;

/// Null-ness of elements in collections.
pub trait Null: Length {
    /// Returns `true` if the element at position `index` is null.
    fn is_null(&self, index: usize) -> Option<bool> {
        self.is_valid(index).map(Not::not)
    }

    /// # Safety
    /// todo(mb)
    unsafe fn is_null_unchecked(&self, index: usize) -> bool {
        !self.is_valid_unchecked(index)
    }

    /// Returns the number of null elements.
    fn null_count(&self) -> usize {
        self.len() - self.valid_count()
    }

    /// Returns `true` if the element at position `index` is valid.
    fn is_valid(&self, index: usize) -> Option<bool>;

    /// # Safety
    /// todo(mb)
    unsafe fn is_valid_unchecked(&self, index: usize) -> bool;

    /// Returns the number of valid elements.
    fn valid_count(&self) -> usize;

    /// Returns `true` if the array contains at least one null element.
    fn any_null(&self) -> bool {
        self.null_count() > 0
    }

    /// Returns `true` if all the elements are null.
    fn all_null(&self) -> bool {
        self.null_count() == self.len()
    }

    /// Returns `true` if the array contains at least one valid element.
    fn any_valid(&self) -> bool {
        self.valid_count() > 0
    }

    /// Returns `true` if all the elements are valid.
    fn all_valid(&self) -> bool {
        self.valid_count() == self.len()
    }
}
