use std::{iter::Map, slice};

use crate::collection::{Collection, CollectionAlloc, view::AsView};

impl<T: for<'any> AsView<'any>> Collection for Box<[T]> {
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

    type IntoIter = <Box<[T]> as IntoIterator>::IntoIter;

    fn into_iter_owned(self) -> Self::IntoIter {
        <Box<[T]> as IntoIterator>::into_iter(self)
    }
}

impl<T: for<'any> AsView<'any>> CollectionAlloc for Box<[T]> {
    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity).into_boxed_slice()
    }
}
