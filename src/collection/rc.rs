extern crate alloc;

use crate::collection::slice::SliceViews;
use alloc::rc::Rc;

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
        = SliceViews<'collection, T>
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        SliceViews::new(self)
    }

    type IntoIter = SliceIntoIter<Self, T>;

    fn into_iter_owned(self) -> Self::IntoIter {
        self.into()
    }
}
