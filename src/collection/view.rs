extern crate alloc;

use crate::{collection::owned::IntoOwned, fixed_size::FixedSize};
use alloc::vec::Vec;
use core::{
    iter::{Copied, FusedIterator, Map},
    slice,
};

/// Convert items into views.
///
/// This is the dual of [`IntoOwned`]: copyable Arrow scalars remain values,
/// while variable-size values become borrowed slices. Collections can
/// therefore expose the cheapest useful item representation through one API.
///
/// # Examples
///
/// ```
/// use narrow::collection::view::AsView;
///
/// let values = vec![1, 2];
/// let view: &[i32] = values.as_view();
/// assert_eq!(view, [1, 2]);
/// ```
pub trait AsView<'collection>: Sized + 'collection {
    /// The view type.
    type View: Copy + IntoOwned<Self> + 'collection;

    /// Iterator over the views of the items in a slice, returned by
    /// [`AsView::views`].
    ///
    /// Slice-backed collections iterate their items through it. The
    /// implementations in this crate use iterators from `core`, such as
    /// `Copied<slice::Iter<'_, Self>>`, which keep the fast paths that `core`
    /// and `alloc` take for iterators of known length, for example when
    /// collecting into or extending a [`Vec`]. Other implementations should do
    /// the same where they can.
    type Views: ExactSizeIterator<Item = Self::View> + DoubleEndedIterator + FusedIterator + Clone;

    /// Returns a view of self.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::view::AsView;
    ///
    /// let values = vec![1, 2];
    /// assert_eq!(values.as_view(), &[1, 2]);
    /// ```
    fn as_view(&'collection self) -> Self::View;

    /// Returns an iterator over the views of the items in `slice`.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::collection::view::AsView;
    ///
    /// let values = [vec![1, 2], vec![3]];
    /// let views = Vec::views(&values);
    /// assert_eq!(views.len(), 2);
    /// assert_eq!(views.rev().collect::<Vec<_>>(), [&[3][..], &[1, 2]]);
    /// ```
    fn views(slice: &'collection [Self]) -> Self::Views;
}

impl<'a> AsView<'a> for bool {
    type View = bool;
    type Views = Copied<slice::Iter<'a, bool>>;

    fn as_view(&'a self) -> bool {
        *self
    }

    fn views(slice: &'a [Self]) -> Self::Views {
        slice.iter().copied()
    }
}

#[diagnostic::do_not_recommend]
impl<'a, T: FixedSize> AsView<'a> for T {
    type View = T;
    type Views = Copied<slice::Iter<'a, T>>;

    fn as_view(&'a self) -> T {
        *self
    }

    fn views(slice: &'a [Self]) -> Self::Views {
        slice.iter().copied()
    }
}

#[diagnostic::do_not_recommend]
impl<'a, T: 'a + Clone, const N: usize> AsView<'a> for [T; N] {
    type View = &'a [T; N];
    type Views = slice::Iter<'a, [T; N]>;

    fn as_view(&'a self) -> &'a [T; N] {
        self
    }

    fn views(slice: &'a [Self]) -> Self::Views {
        slice.iter()
    }
}

impl<'collection, T: Clone + 'collection> AsView<'collection> for Vec<T> {
    type View = &'collection [T];
    type Views = Map<slice::Iter<'collection, Self>, fn(&'collection Self) -> &'collection [T]>;

    fn as_view(&'collection self) -> Self::View {
        self
    }

    fn views(slice: &'collection [Self]) -> Self::Views {
        let as_slice: fn(&'collection Self) -> &'collection [T] = Self::as_slice;
        slice.iter().map(as_slice)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use crate::collection::Collection;
    use alloc::vec;

    use super::*;

    #[test]
    #[expect(clippy::perf, clippy::unwrap_used)]
    fn view() {
        let a = vec![1, 2, 3, 4];
        assert_eq!(a[0].as_view(), 1);

        let b = [1, 2, 3, 4];
        assert_eq!(b.as_view(), &[1, 2, 3, 4]);

        let c = [1, 2, 3, 4].as_slice();
        assert_eq!(c.view(2), Some(3));

        let d = vec![vec![1, 2], vec![3, 4]];
        assert_eq!(d.view(0), Some([1, 2].as_slice()));
        assert_eq!(d.view(1).unwrap().view(1), Some(4));
    }
}
