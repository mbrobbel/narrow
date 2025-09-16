//! Collections of items.

use std::{array, borrow::Borrow, iter::Map, marker::PhantomData, rc::Rc, slice, sync::Arc, vec};

use crate::length::Length;

/// Convert into owned items.
pub trait IntoOwned<Owned> {
    /// Returns the owned instance of self.
    fn into_owned(self) -> Owned;
}

impl<T> IntoOwned<T> for T {
    fn into_owned(self) -> T {
        self
    }
}

impl<T: Clone> IntoOwned<T> for &T {
    fn into_owned(self) -> T {
        self.clone()
    }
}

impl<T: Clone> IntoOwned<Vec<T>> for &[T] {
    fn into_owned(self) -> Vec<T> {
        self.to_vec()
    }
}

impl<T: Clone> IntoOwned<Box<[T]>> for &[T] {
    fn into_owned(self) -> Box<[T]> {
        self.to_vec().into_boxed_slice()
    }
}

impl<T: Clone> IntoOwned<Rc<[T]>> for &[T] {
    fn into_owned(self) -> Rc<[T]> {
        Rc::<[T]>::from(self.to_vec().into_boxed_slice())
    }
}

impl<T: Clone> IntoOwned<Arc<[T]>> for &[T] {
    fn into_owned(self) -> Arc<[T]> {
        Arc::<[T]>::from(self.to_vec().into_boxed_slice())
    }
}

/// Convert items into views.
pub trait AsView<'collection> {
    /// The view type.
    type View: 'collection;

    /// Returns a view of self.
    fn as_view(&'collection self) -> Self::View;
}

impl<'collection, T: 'collection> AsView<'collection> for Vec<T> {
    type View = &'collection [T];

    fn as_view(&'collection self) -> Self::View {
        self
    }
}

impl<'collection, T: 'collection> AsView<'collection> for &[T] {
    type View = &'collection [T];

    fn as_view(&'collection self) -> Self::View {
        self
    }
}

/// The view type of T.
pub type ViewOf<'collection, T> = <T as AsView<'collection>>::View;

/// A collection of items.
pub trait Collection: Length {
    /// Borrowed view of an item in this collection
    type View<'collection>
    where
        Self: 'collection;

    /// Owned items in this collection
    type Owned;

    /// Returns a reference to an item at the given index in this collection or
    /// `None` if out of bounds.
    fn view(&self, index: usize) -> Option<Self::View<'_>>;

    /// Returns an owned item at the given index in this collection or `None`
    /// if out of bounds.
    fn owned(&self, index: usize) -> Option<Self::Owned>
    where
        for<'collection> Self::View<'collection>: IntoOwned<Self::Owned>,
    {
        self.view(index).map(IntoOwned::into_owned)
    }

    /// Iterator over referenced items in this collection.
    type Iter<'collection>: Iterator<Item = Self::View<'collection>>
    where
        Self: 'collection;

    /// Returns an iterator over references to the items in this collection.
    fn iter(&self) -> Self::Iter<'_>;

    /// Iterator over owned items in this collection.
    type IntoIter: Iterator<Item = Self::Owned>;

    /// Returns an interator over items in this collection.
    fn into_iter(self) -> Self::IntoIter;
}

/// An allocatable collection of items.
pub trait CollectionAlloc: Collection + Default + FromIterator<Self::Owned> {
    /// Constructs a new, empty collection with at least the specified capacity.
    fn with_capacity(capacity: usize) -> Self;
}

/// A re-allocatable collection of items.
pub trait CollectionRealloc: CollectionAlloc + Extend<Self::Owned> {
    /// Reserves capacity for at least `additional` more items to be inserted in this collection.
    fn reserve(&mut self, additional: usize);
}

impl<T> Collection for Vec<T>
where
    for<'collection> T: AsView<'collection>,
{
    type View<'collection>
        = ViewOf<'collection, T>
    where
        Self: 'collection;

    type Owned = T;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.get(index).map(AsView::as_view)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> ViewOf<'collection, T>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self).map(AsView::as_view)
    }

    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<T> CollectionAlloc for Vec<T>
where
    for<'collection> T: AsView<'collection>,
{
    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity)
    }
}

impl<T> CollectionRealloc for Vec<T>
where
    for<'collection> T: AsView<'collection>,
{
    fn reserve(&mut self, additional: usize) {
        Vec::reserve(self, additional);
    }
}

impl<T, const N: usize> Collection for [T; N]
where
    for<'collection> T: AsView<'collection>,
{
    type View<'collection>
        = ViewOf<'collection, T>
    where
        Self: 'collection;

    type Owned = T;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.get(index).map(AsView::as_view)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> ViewOf<'collection, T>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self).map(AsView::as_view)
    }

    type IntoIter = array::IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

/// An iterator over a slice that turns views into owned instances
#[derive(Debug)]
pub struct SliceIntoIter<T, U> {
    /// The item that can be borrowed as slice
    slice: T,
    /// The current index
    index: usize,
    /// Item type
    _ty: PhantomData<U>,
}

impl<T: Borrow<[U]>, U: for<'slice> AsView<'slice, View: IntoOwned<U>>> Iterator
    for SliceIntoIter<T, U>
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        self.slice
            .borrow()
            .get(self.index)
            .inspect(|_| {
                self.index += 1;
            })
            .map(|item| item.as_view().into_owned())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.slice.borrow().len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<T: Borrow<[U]>, U: for<'slice> AsView<'slice, View: IntoOwned<U>>> ExactSizeIterator
    for SliceIntoIter<T, U>
{
}

impl<T, U> From<T> for SliceIntoIter<T, U> {
    fn from(slice: T) -> Self {
        Self {
            slice,
            index: 0,
            _ty: PhantomData,
        }
    }
}

impl<T> Collection for &[T]
where
    for<'collection> T: AsView<'collection, View: IntoOwned<T>>,
{
    type View<'collection>
        = ViewOf<'collection, T>
    where
        Self: 'collection;

    type Owned = T;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.get(index).map(AsView::as_view)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> ViewOf<'collection, T>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <Self as IntoIterator>::into_iter(self).map(AsView::as_view)
    }

    type IntoIter = SliceIntoIter<Self, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

impl<T> Collection for Box<[T]>
where
    for<'collection> T: AsView<'collection>,
{
    type View<'collection>
        = ViewOf<'collection, T>
    where
        Self: 'collection;

    type Owned = T;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.get(index).map(AsView::as_view)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> ViewOf<'collection, T>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self).map(AsView::as_view)
    }

    type IntoIter = <Box<[T]> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <Box<[T]> as IntoIterator>::into_iter(self)
    }
}

impl<T> CollectionAlloc for Box<[T]>
where
    for<'collection> T: AsView<'collection>,
{
    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity).into_boxed_slice()
    }
}

impl<T> Collection for Rc<[T]>
where
    for<'collection> T: AsView<'collection, View: IntoOwned<T>>,
{
    type View<'collection>
        = ViewOf<'collection, T>
    where
        Self: 'collection;

    type Owned = T;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.get(index).map(AsView::as_view)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> ViewOf<'collection, T>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T] as IntoIterator>::into_iter(self).map(AsView::as_view)
    }

    type IntoIter = SliceIntoIter<Self, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

impl<T> Collection for Arc<[T]>
where
    for<'collection> T: AsView<'collection, View: IntoOwned<T>>,
{
    type View<'collection>
        = ViewOf<'collection, T>
    where
        Self: 'collection;

    type Owned = T;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.get(index).map(AsView::as_view)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> ViewOf<'collection, T>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T] as IntoIterator>::into_iter(self).map(AsView::as_view)
    }

    type IntoIter = SliceIntoIter<Self, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection() {
        let a = vec![1, 2, 3, 4];
        assert_eq!(a[0].as_view(), 1);

        let b = [1, 2, 3, 4];
        assert_eq!(b.as_view(), [1, 2, 3, 4]);

        let c = [1, 2, 3, 4].as_slice();
        assert_eq!(c.as_view(), c);
        assert_eq!(c.view(2), Some(3));

        let d = vec![vec![1, 2], vec![3, 4]];
        assert_eq!(d.view(0), Some([1, 2].as_slice()));
        assert_eq!(d.view(1).unwrap().view(1), Some(4));
    }

    #[test]
    fn slice_into_iter_size_hint() {
        let input = [1, 2, 3, 4].as_slice();
        let mut iter = SliceIntoIter::from(input);
        assert_eq!(iter.size_hint(), (4, Some(4)));
        let _ = Iterator::next(&mut iter);
        assert_eq!(iter.size_hint(), (3, Some(3)));
        let _ = iter.nth(2);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        let _ = Iterator::next(&mut iter);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }
}
