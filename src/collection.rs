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
// can't add Default as super trait because generic arrays don't implement
// default
pub trait Item: Sized + 'static {
    /// A reference type for this item when stored in a collection.
    type Ref<'collection>;

    /// Borrow this items as [`Item::Ref`].
    fn as_ref(&self) -> Self::Ref<'_>;

    /// Converts a reference to [`Item::Ref`] to an owned [`Item`].
    fn to_owned(item: &Self::Ref<'_>) -> Self;

    /// Converts [`Item::Ref`] into an owned [`Item`].
    fn into_owned(item: Self::Ref<'_>) -> Self {
        <Self as Item>::to_owned(&item)
    }
}

/// A collection of items `T`.
pub trait Collection: Length {
    /// The item stored in this collection.
    type Item: Item;

    /// A reference type for items in this collection.
    /// This typically defaults to <Self::Item as Item>::Ref<'collection>.
    type Ref<'collection>
    where
        Self: 'collection;

    /// Returns a reference to an item in this collection or `None` if out of bounds.
    fn index(&self, index: usize) -> Option<Self::Ref<'_>>;

    /// Iterator over referenced items in this collection.
    type Iter<'collection>: Iterator<Item = Self::Ref<'collection>>
    where
        Self: 'collection;

    /// Returns an iterator over references to the items in this collection.
    fn iter(&self) -> Self::Iter<'_>;

    /// Iterator over items in this collection.
    type IntoIter: Iterator<Item = Self::Item>;

    /// Returns an interator over items in this collection.
    fn into_iter(self) -> Self::IntoIter;
}

/// An allocatable collection of items.
pub trait CollectionAlloc: Collection + Default + FromIterator<Self::Item> {
    /// Constructs a new, empty collection with at least the specified capacity.
    fn with_capacity(capacity: usize) -> Self;
}

/// A re-allocatable collection of items.
pub trait CollectionRealloc: CollectionAlloc + Extend<Self::Item> {
    /// Reserves capacity for at least `additional` more items to be inserted in this collection.
    fn reserve(&mut self, additional: usize);
}

impl<T: Item> Collection for Vec<T> {
    type Item = T;
    type Ref<'collection> = T::Ref<'collection>;

    fn index(&self, index: usize) -> Option<Self::Ref<'_>> {
        self.get(index).map(T::as_ref)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> Self::Ref<'collection>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self).map(T::as_ref)
    }

    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<T: Item> CollectionAlloc for Vec<T> {
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }
}

impl<T: Item> CollectionRealloc for Vec<T> {
    fn reserve(&mut self, additional: usize) {
        Self::reserve(self, additional);
    }
}

impl<T: Item, const N: usize> Collection for [T; N] {
    type Item = T;
    type Ref<'collection> = T::Ref<'collection>;

    fn index(&self, index: usize) -> Option<Self::Ref<'_>> {
        self.get(index).map(T::as_ref)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> Self::Ref<'collection>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self).map(Self::Item::as_ref)
    }

    type IntoIter = array::IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<'a, T: Copy + Item> Collection for &'a [T] {
    type Item = T;
    type Ref<'collection>
        = T::Ref<'collection>
    where
        Self: 'collection;

    fn index(&self, index: usize) -> Option<Self::Ref<'_>> {
        self.get(index).map(T::as_ref)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> Self::Ref<'collection>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <Self as IntoIterator>::into_iter(self).map(T::as_ref)
    }

    type IntoIter = Copied<slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self).copied()
    }
}

impl<T: Item> Collection for Box<[T]> {
    type Item = T;
    type Ref<'collection> = T::Ref<'collection>;

    fn index(&self, index: usize) -> Option<Self::Ref<'_>> {
        self.get(index).map(T::as_ref)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> Self::Ref<'collection>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self).map(T::as_ref)
    }

    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<T: Item> CollectionAlloc for Box<[T]> {
    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity).into_boxed_slice()
    }
}

/// An iterator over copied items `T` in a slice borrowed from `U`.
#[derive(Debug)]
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

impl<T: Copy + Item> Collection for Rc<[T]> {
    type Item = T;
    type Ref<'collection> = T::Ref<'collection>;

    fn index(&self, index: usize) -> Option<Self::Ref<'_>> {
        self.get(index).map(T::as_ref)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> Self::Ref<'collection>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T] as IntoIterator>::into_iter(self).map(T::as_ref)
    }

    type IntoIter = CopySliceIter<T, Self>;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

impl<T: Copy + Item> Collection for Arc<[T]> {
    type Item = T;
    type Ref<'collection> = T::Ref<'collection>;

    fn index(&self, index: usize) -> Option<Self::Ref<'_>> {
        self.get(index).map(T::as_ref)
    }

    type Iter<'collection>
        = Map<slice::Iter<'collection, T>, fn(&'collection T) -> Self::Ref<'collection>>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T] as IntoIterator>::into_iter(self).map(T::as_ref)
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
