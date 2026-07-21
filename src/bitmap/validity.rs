//! Validity information stored in a bitmap.

use super::BitmapRef;
use crate::collection::Collection;

/// A bitmap storing the validity of elements in a collection.
pub trait ValidityBitmap: BitmapRef {
    /// Returns whether the element at `index` is null, or [`None`] when the
    /// index is out of bounds.
    fn is_null(&self, index: usize) -> Option<bool> {
        self.is_valid(index).map(|valid| !valid)
    }

    /// Returns the number of null elements.
    fn null_count(&self) -> usize {
        self.bitmap_ref()
            .iter_views()
            .filter(|valid| !*valid)
            .count()
    }

    /// Returns whether the element at `index` is valid, or [`None`] when the
    /// index is out of bounds.
    fn is_valid(&self, index: usize) -> Option<bool> {
        self.bitmap_ref().view(index)
    }

    /// Returns the number of valid elements.
    fn valid_count(&self) -> usize {
        self.bitmap_ref()
            .iter_views()
            .filter(|valid| *valid)
            .count()
    }

    /// Returns whether the collection contains at least one null element.
    fn any_null(&self) -> bool {
        self.bitmap_ref().iter_views().any(|valid| !valid)
    }

    /// Returns whether all elements in the collection are null.
    fn all_null(&self) -> bool {
        self.bitmap_ref().iter_views().all(|valid| !valid)
    }

    /// Returns whether the collection contains at least one valid element.
    fn any_valid(&self) -> bool {
        self.bitmap_ref().iter_views().any(|valid| valid)
    }

    /// Returns whether all elements in the collection are valid.
    fn all_valid(&self) -> bool {
        self.bitmap_ref().iter_views().all(|valid| valid)
    }
}
