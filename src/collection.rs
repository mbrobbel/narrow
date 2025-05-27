//! Collections of items.

use std::{
    array,
    borrow::Borrow,
    iter::{Copied, Map},
    marker::PhantomData,
    rc::Rc,
    slice,
    sync::Arc,
    vec,
};

use crate::length::Length;

/// An item that can be stored in a [`Collection`].
pub trait Item: Sized + 'static {
    /// A reference type for this item when stored in a collection.
    type RefItem<'collection>;

    /// Borrow this items as [`Item::RefItem`].
    fn as_ref_item(&self) -> Self::RefItem<'_>;

    /// Converts a reference to [`Item::RefItem`] to an owned [`Item`].
    fn to_owned(item: &Self::RefItem<'_>) -> Self;

    /// Converts [`Item::RefItem`] into an owned [`Item`].
    fn into_owned(item: Self::RefItem<'_>) -> Self {
        <Self as Item>::to_owned(&item)
    }
}

/// A collection of items `T`.
pub trait Collection<T: Item>: Length {
    /// Returns a reference to an item in this collection or `None` if out of bounds.
    fn index(&self, index: usize) -> Option<T::RefItem<'_>>;

    /// Iterator over referenced items in this collection.
    type Iter<'collection>: Iterator<Item = T::RefItem<'collection>>
    where
        Self: 'collection;

    /// Returns an iterator over references to the items in this collection.
    fn iter(&self) -> Self::Iter<'_>;

    /// Iterator over items in this collection.
    type IntoIter: Iterator<Item = T>;

    /// Returns an interator over items in this collection.
    fn into_iter(self) -> Self::IntoIter;
}

/// An allocatable collection of items.
pub trait CollectionAlloc<T: Item>: Collection<T> + Default + FromIterator<T> {
    /// Constructs a new, empty collection with at least the specified capacity.
    fn with_capacity(capacity: usize) -> Self;
}

/// A re-allocatable collection of items.
pub trait CollectionRealloc<T: Item>: CollectionAlloc<T> + Extend<T> {
    /// Reserves capacity for at least `additional` more items to be inserted in this collection.
    fn reserve(&mut self, additional: usize);
}

impl<T: Item> Collection<T> for Vec<T> {
    fn index(&self, index: usize) -> Option<T::RefItem<'_>> {
        self.get(index).map(T::as_ref_item)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> T::RefItem<'collection>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self).map(T::as_ref_item)
    }

    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<T: Item> CollectionAlloc<T> for Vec<T> {
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }
}

impl<T: Item> CollectionRealloc<T> for Vec<T> {
    fn reserve(&mut self, additional: usize) {
        Self::reserve(self, additional);
    }
}

impl<T: Item, const N: usize> Collection<T> for [T; N] {
    fn index(&self, index: usize) -> Option<T::RefItem<'_>> {
        self.get(index).map(T::as_ref_item)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> T::RefItem<'collection>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self).map(T::as_ref_item)
    }

    type IntoIter = array::IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<'a, T: Copy + Item> Collection<T> for &'a [T] {
    fn index(&self, index: usize) -> Option<T::RefItem<'_>> {
        self.get(index).map(T::as_ref_item)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> T::RefItem<'collection>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <Self as IntoIterator>::into_iter(self).map(T::as_ref_item)
    }

    type IntoIter = Copied<slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self).copied()
    }
}

impl<T: Item> Collection<T> for Box<[T]> {
    fn index(&self, index: usize) -> Option<T::RefItem<'_>> {
        self.get(index).map(T::as_ref_item)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> T::RefItem<'collection>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self).map(T::as_ref_item)
    }

    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<T: Item> CollectionAlloc<T> for Box<[T]> {
    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity).into_boxed_slice()
    }
}

/// An iterator over copied items `T` in a slice borrowed from `U`.
pub struct CopySliceIter<T: Copy, U: Borrow<[T]>> {
    /// The slice is borrowed from this field.
    data: U,
    /// The current position of this iterator.
    index: usize,
    /// The item type.
    _ty: PhantomData<T>,
}

impl<T: Copy, U: Borrow<[T]>> From<U> for CopySliceIter<T, U> {
    fn from(data: U) -> Self {
        Self {
            data,
            index: 0,
            _ty: PhantomData,
        }
    }
}

impl<T: Copy, U: Borrow<[T]>> Iterator for CopySliceIter<T, U> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.data
            .borrow()
            .get(self.index)
            .inspect(|_| {
                self.index += 1;
            })
            .copied()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.data.borrow().len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<T: Copy, U: Borrow<[T]>> ExactSizeIterator for CopySliceIter<T, U> {}

impl<T: Copy + Item> Collection<T> for Rc<[T]> {
    fn index(&self, index: usize) -> Option<T::RefItem<'_>> {
        self.get(index).map(T::as_ref_item)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> T::RefItem<'collection>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T] as IntoIterator>::into_iter(self).map(T::as_ref_item)
    }

    type IntoIter = CopySliceIter<T, Self>;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

impl<T: Copy + Item> Collection<T> for Arc<[T]> {
    fn index(&self, index: usize) -> Option<T::RefItem<'_>> {
        self.get(index).map(T::as_ref_item)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> T::RefItem<'collection>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T] as IntoIterator>::into_iter(self).map(T::as_ref_item)
    }

    type IntoIter = CopySliceIter<T, Self>;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrow_slice_into_iter_size_hint() {
        let input = [1, 2, 3, 4].as_slice();
        let mut iter = CopySliceIter::from(input);
        assert_eq!(iter.size_hint(), (4, Some(4)));
        let _ = Iterator::next(&mut iter);
        assert_eq!(iter.size_hint(), (3, Some(3)));
        let _ = iter.nth(2);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        let _ = Iterator::next(&mut iter);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }
}
