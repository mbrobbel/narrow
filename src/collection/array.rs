use std::{array, iter::Map, slice};

use crate::collection::{Collection, view::AsView};

impl<T: for<'any> AsView<'any>, const N: usize> Collection for [T; N] {
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

    type IntoIter = array::IntoIter<T, N>;

    fn into_iter_owned(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}
