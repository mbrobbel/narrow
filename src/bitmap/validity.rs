//! Validity information stored in a bitmap.

use super::BitmapRef;
use crate::collection::Collection;

/// A bitmap storing the validity of elements in a collection.
///
/// Layouts that carry nullability all expose the same semantic questions even
/// when their value buffers differ. This trait centralizes those queries over
/// [`BitmapRef`] so callers do not need to interpret validity bits themselves.
///
/// # Examples
///
/// ```
/// use narrow::{bitmap::ValidityBitmap, validity::Validity};
///
/// let values = [Some(1), None].into_iter().collect::<Validity<Vec<i32>>>();
/// assert_eq!((values.valid_count(), values.null_count()), (1, 1));
/// ```
pub trait ValidityBitmap: BitmapRef {
    /// Returns whether the element at `index` is null, or [`None`] when the
    /// index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{bitmap::ValidityBitmap, validity::Validity};
    ///
    /// let values = [Some(1), None].into_iter().collect::<Validity<Vec<i32>>>();
    /// assert_eq!(values.is_null(1), Some(true));
    /// ```
    fn is_null(&self, index: usize) -> Option<bool> {
        self.is_valid(index).map(|valid| !valid)
    }

    /// Returns the number of null elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{bitmap::ValidityBitmap, validity::Validity};
    ///
    /// let values = [Some(1), None].into_iter().collect::<Validity<Vec<i32>>>();
    /// assert_eq!(values.null_count(), 1);
    /// ```
    fn null_count(&self) -> usize {
        self.bitmap_ref()
            .iter_views()
            .filter(|valid| !*valid)
            .count()
    }

    /// Returns whether the element at `index` is valid, or [`None`] when the
    /// index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{bitmap::ValidityBitmap, validity::Validity};
    ///
    /// let values = [Some(1), None].into_iter().collect::<Validity<Vec<i32>>>();
    /// assert_eq!(values.is_valid(0), Some(true));
    /// ```
    fn is_valid(&self, index: usize) -> Option<bool> {
        self.bitmap_ref().view(index)
    }

    /// Returns the number of valid elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{bitmap::ValidityBitmap, validity::Validity};
    ///
    /// let values = [Some(1), None].into_iter().collect::<Validity<Vec<i32>>>();
    /// assert_eq!(values.valid_count(), 1);
    /// ```
    fn valid_count(&self) -> usize {
        self.bitmap_ref()
            .iter_views()
            .filter(|valid| *valid)
            .count()
    }

    /// Returns whether the collection contains at least one null element.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{bitmap::ValidityBitmap, validity::Validity};
    ///
    /// let values = [Some(1), None].into_iter().collect::<Validity<Vec<i32>>>();
    /// assert!(values.any_null());
    /// ```
    fn any_null(&self) -> bool {
        self.bitmap_ref().iter_views().any(|valid| !valid)
    }

    /// Returns whether all elements in the collection are null.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{bitmap::ValidityBitmap, validity::Validity};
    ///
    /// let values = [None, None].into_iter().collect::<Validity<Vec<i32>>>();
    /// assert!(values.all_null());
    /// ```
    fn all_null(&self) -> bool {
        self.bitmap_ref().iter_views().all(|valid| !valid)
    }

    /// Returns whether the collection contains at least one valid element.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{bitmap::ValidityBitmap, validity::Validity};
    ///
    /// let values = [None, Some(1)].into_iter().collect::<Validity<Vec<i32>>>();
    /// assert!(values.any_valid());
    /// ```
    fn any_valid(&self) -> bool {
        self.bitmap_ref().iter_views().any(|valid| valid)
    }

    /// Returns whether all elements in the collection are valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{bitmap::ValidityBitmap, validity::Validity};
    ///
    /// let values = [Some(1), Some(2)].into_iter().collect::<Validity<Vec<i32>>>();
    /// assert!(values.all_valid());
    /// ```
    fn all_valid(&self) -> bool {
        self.bitmap_ref().iter_views().all(|valid| valid)
    }
}
