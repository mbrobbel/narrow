extern crate alloc;

use crate::collection::slice::views_range;
use alloc::rc::Rc;
use core::ops::Range;

use crate::collection::{Collection, slice::SliceIntoIter, view::AsView};

impl<T: for<'any> AsView<'any>> Collection for Rc<[T]> {
    type View<'collection>
        = <T as AsView<'collection>>::View
    where
        Self: 'collection;

    type Owned = T;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.get(index).map(AsView::as_view)
    }

    type Iter<'collection>
        = <T as AsView<'collection>>::Views
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        T::views(self)
    }

    fn iter_views_range(&self, range: Range<usize>) -> Self::Iter<'_> {
        views_range(self, range)
    }

    type IntoIter = SliceIntoIter<Self, T>;

    fn into_iter_owned(self) -> Self::IntoIter {
        self.into()
    }
}
