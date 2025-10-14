use std::{iter::Map, slice, vec};

use crate::collection::{Collection, CollectionAlloc, CollectionRealloc, view::AsView};

impl<T: for<'any> AsView<'any>> Collection for Vec<T> {
    type View<'collection>
        = <T as AsView<'collection>>::View
    where
        Self: 'collection;

    type Owned = T;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.get(index).map(AsView::as_view)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> <T as AsView<'collection>>::View>
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self).map(AsView::as_view)
    }

    type IntoIter = vec::IntoIter<T>;

    fn into_iter_owned(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<T: for<'any> AsView<'any>> CollectionAlloc for Vec<T> {
    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity)
    }
}

impl<T: for<'any> AsView<'any>> CollectionRealloc for Vec<T> {
    fn reserve(&mut self, additional: usize) {
        Vec::reserve(self, additional);
    }
}
