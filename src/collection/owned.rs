extern crate alloc;

use crate::fixed_size::FixedSize;
use alloc::vec::Vec;

/// Convert into owned items.
///
/// Collection views may borrow nested data even when their owned item is a
/// container. This trait gives generic code one explicit boundary at which to
/// materialize that ownership.
///
/// # Examples
///
/// ```
/// use narrow::collection::owned::IntoOwned;
///
/// let owned: Vec<_> = (&[1, 2][..]).into_owned();
/// assert_eq!(owned, [1, 2]);
/// ```
pub trait IntoOwned<Owned> {
    /// Returns the owned instance of self.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::owned::IntoOwned;
    ///
    /// assert_eq!((&[1, 2][..]).into_owned(), vec![1, 2]);
    /// ```
    fn into_owned(self) -> Owned;
}

impl IntoOwned<bool> for bool {
    fn into_owned(self) -> bool {
        self
    }
}

#[diagnostic::do_not_recommend]
impl<T: FixedSize> IntoOwned<T> for T {
    fn into_owned(self) -> T {
        self
    }
}

impl<T: Clone, const N: usize> IntoOwned<[T; N]> for &[T; N] {
    fn into_owned(self) -> [T; N] {
        self.clone()
    }
}

impl<T: Clone> IntoOwned<Vec<T>> for &[T] {
    fn into_owned(self) -> Vec<T> {
        self.to_vec()
    }
}

impl<T: IntoOwned<U>, U> IntoOwned<Option<U>> for Option<T> {
    fn into_owned(self) -> Option<U> {
        self.map(IntoOwned::into_owned)
    }
}
