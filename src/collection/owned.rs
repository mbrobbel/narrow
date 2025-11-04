extern crate alloc;

use crate::fixed_size::FixedSize;
use alloc::vec::Vec;

/// Convert into owned items.
pub trait IntoOwned<Owned> {
    /// Returns the owned instance of self.
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
