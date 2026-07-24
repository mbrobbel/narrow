//! Optional validity information stored in a bitmap.

use super::Bitmap;
use crate::{buffer::Buffer, collection::Collection, length::Length};

/// Optional bitmap storage for the validity of elements in a collection.
///
/// Layouts that carry nullability all expose the same semantic questions even
/// when their value buffers differ. This trait centralizes those queries so
/// callers do not need to interpret validity bits themselves.
///
/// # Examples
///
/// ```
/// use narrow::{bitmap::ValidityBitmap, validity::Validity};
///
/// let values = [Some(1), None].into_iter().collect::<Validity<Vec<i32>>>();
/// assert_eq!((values.valid_count(), values.null_count()), (1, 1));
/// ```
pub trait ValidityBitmap: Length {
    /// Storage of the validity bitmap.
    type Storage: Buffer;

    /// Returns the validity bitmap, or [`None`] when all items are valid.
    ///
    /// Arrow permits arrays without a validity bitmap when their null count is
    /// zero. See the [Arrow validity bitmap specification].
    ///
    /// [Arrow validity bitmap specification]: https://arrow.apache.org/docs/format/Columnar.html#validity-bitmaps
    fn bitmap_ref(&self) -> Option<&Bitmap<Self::Storage>>;

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
        self.bitmap_ref().map_or(0, |bitmap| {
            bitmap.iter_views().filter(|valid| !*valid).count()
        })
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
        (index < self.len()).then(|| {
            self.bitmap_ref()
                .is_none_or(|bitmap| bitmap.view(index).expect("validity lengths match"))
        })
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
        self.len().strict_sub(self.null_count())
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
        self.bitmap_ref()
            .is_some_and(|bitmap| bitmap.iter_views().any(|valid| !valid))
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
        self.null_count() == self.len()
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
        self.valid_count() != 0
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
        self.null_count() == 0
    }
}
