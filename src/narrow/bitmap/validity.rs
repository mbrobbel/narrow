//! Store validity information in a bitmap.

use super::BitmapRef;
use crate::length::Length;
use std::ops::Not;

/// A validity bitmap storing the validity information (null-ness) of elements
/// in a collection in a bitmap.
pub trait ValidityBitmap: BitmapRef {
    /// Returns `true` if the element at position `index` is null.
    #[inline]
    fn is_null(&self, index: usize) -> Option<bool> {
        self.is_valid(index).map(Not::not)
    }

    /// Returns `true` if the element at position `index` is null, without
    /// performing any bounds checking.
    ///
    /// # Safety
    /// - The `index` must be in bounds.
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior.
    #[inline]
    unsafe fn is_null_unchecked(&self, index: usize) -> bool {
        !self.is_valid_unchecked(index)
    }

    /// Returns the number of null elements.
    #[inline]
    fn null_count(&self) -> usize {
        self.bitmap_ref()
            .len()
            .checked_sub(self.valid_count())
            .expect("null count underflow")
    }

    /// Returns `true` if the element at position `index` is valid.
    #[inline]
    fn is_valid(&self, index: usize) -> Option<bool> {
        (index < self.bitmap_ref().len()).then(||
            // Safety:
            // - Bound checked
            unsafe { self.is_valid_unchecked(index) })
    }

    /// Returns `true` if the element at position `index` is valid, without
    /// performing any bounds checking.
    ///
    /// # Safety
    /// - The `index` must be in bounds.
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior.
    #[inline]
    unsafe fn is_valid_unchecked(&self, index: usize) -> bool {
        self.bitmap_ref().get_unchecked(index)
    }

    /// Returns the number of valid elements.
    #[inline]
    fn valid_count(&self) -> usize {
        (0..self.bitmap_ref().len())
            .filter(|&index|
                // Safety:
                // - The index is always in range by iterating over the range
                //   with length upper bound.
                unsafe { self.is_valid_unchecked(index) })
            .count()
    }

    /// Returns `true` if the array contains at least one null element.
    #[inline]
    fn any_null(&self) -> bool {
        self.null_count() > 0
    }

    /// Returns `true` if all the elements are null.
    #[inline]
    fn all_null(&self) -> bool {
        self.null_count() == self.bitmap_ref().len()
    }

    /// Returns `true` if the array contains at least one valid element.
    #[inline]
    fn any_valid(&self) -> bool {
        self.valid_count() > 0
    }

    /// Returns `true` if all the elements are valid.
    #[inline]
    fn all_valid(&self) -> bool {
        self.valid_count() == self.bitmap_ref().len()
    }
}
