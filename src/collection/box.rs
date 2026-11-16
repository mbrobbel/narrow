extern crate alloc;

use crate::collection::slice::SliceViews;
use alloc::{boxed::Box, vec::Vec};
use core::alloc::Allocator;

use crate::collection::{
    Collection, CollectionAllocIn, TryReserveError, vec::try_reserve_exact, view::AsView,
};

impl<T: for<'any> AsView<'any>, A: Allocator> Collection for Box<[T], A> {
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

    type IntoIter = <Self as IntoIterator>::IntoIter;

    fn into_iter_owned(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<T: for<'any> AsView<'any>, A: Allocator + Clone> CollectionAllocIn for Box<[T], A> {
    type Alloc = A;

    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self {
        Vec::with_capacity_in(capacity, alloc).into_boxed_slice()
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, alloc: Self::Alloc) -> Self {
        <Vec<T, A> as CollectionAllocIn>::from_iter_in(iter, alloc).into_boxed_slice()
    }

    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, TryReserveError> {
        <Vec<T, A> as CollectionAllocIn>::try_with_capacity_in(capacity, alloc)
            .map(Vec::into_boxed_slice)
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, TryReserveError> {
        <Vec<T, A> as CollectionAllocIn>::try_from_iter_in(iter, alloc)
            .and_then(try_into_boxed_slice)
    }
}

/// Converts `values` into a boxed slice without shrinking it in place, which
/// aborts when the allocator fails, by moving spare-capacity vectors into an
/// exact reservation first.
fn try_into_boxed_slice<T, A: Allocator + Clone>(
    values: Vec<T, A>,
) -> Result<Box<[T], A>, TryReserveError> {
    if size_of::<T>() == 0 || values.len() == values.capacity() {
        return Ok(values.into_boxed_slice());
    }
    let mut exact = Vec::new_in(Vec::allocator(&values).clone());
    try_reserve_exact(&mut exact, values.len())?;
    exact.extend(values);
    Ok(exact.into_boxed_slice())
}
