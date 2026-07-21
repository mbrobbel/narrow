extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::{iter::Map, slice};

use crate::collection::{AllocError, Collection, CollectionAlloc, CollectionAllocIn, view::AsView};

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

impl<T: for<'any> AsView<'any>> CollectionAllocIn for Box<[T]> {
    type Alloc = ();

    fn with_capacity_in(capacity: usize, (): Self::Alloc) -> Self {
        Vec::with_capacity(capacity).into_boxed_slice()
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, (): Self::Alloc) -> Self {
        iter.into_iter().collect::<Vec<_>>().into_boxed_slice()
    }

    fn try_with_capacity_in(capacity: usize, (): Self::Alloc) -> Result<Self, AllocError> {
        <Vec<T> as CollectionAllocIn>::try_with_capacity_in(capacity, ()).map(Vec::into_boxed_slice)
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        (): Self::Alloc,
    ) -> Result<Self, AllocError> {
        <Vec<T> as CollectionAllocIn>::try_from_iter_in(iter, ()).map(Vec::into_boxed_slice)
    }
}
