//! Collections of items.

use std::{
    array, borrow::Borrow, iter::Copied, marker::PhantomData, rc::Rc, slice, sync::Arc, vec,
};

use crate::length::Length;

/// A collection of items.
pub trait Collection: Length {
    /// The items in this collection.
    type Item;

    /// Reference type of items in this collection.
    type RefItem<'collection>
    where
        Self: 'collection;

    /// Returns a reference to an item in this collection or `None` if out of bounds.
    fn index(&self, index: usize) -> Option<Self::RefItem<'_>>;

    /// Iterator over referenced items in this collection.
    type Iter<'collection>: Iterator<Item = Self::RefItem<'collection>>
    where
        Self: 'collection;

    /// Returns an iterator over references to the items in this collection.
    fn iter(&self) -> Self::Iter<'_>;

    /// Iterator over items in this collection.
    type IntoIter: Iterator<Item = Self::Item>;

    /// Returns an interator over items in this collection.
    fn into_iter(self) -> Self::IntoIter;
}

/// A mutable collection of items.
pub trait CollectionMut: Collection {
    /// Reference type of mutable items in this collection.
    type RefItemMut<'collection>
    where
        Self: 'collection;

    /// Returns a mutable reference to an item in this collection or `None` if out of bounds.
    fn index_mut(&mut self, index: usize) -> Option<Self::RefItemMut<'_>>;

    /// Iterator over referenced mutable items in this collection.
    type IterMut<'collection>: Iterator<Item = Self::RefItemMut<'collection>>
    where
        Self: 'collection;

    /// Returns an iterator over mutable references to the items in this collection.
    fn iter_mut(&mut self) -> Self::IterMut<'_>;
}

/// An allocatable collection of items.
pub trait CollectionAlloc:
    Collection + Default + Extend<Self::Item> + FromIterator<Self::Item>
{
    /// Constructs a new, empty collection with at least the specified capacity.
    fn with_capacity(capacity: usize) -> Self;

    /// Reserves capacity for at least `additional` more items to be inserted in this collection.
    fn reserve(&mut self, additional: usize);
}

impl<T> Collection for Vec<T> {
    type Item = T;

    type RefItem<'collection>
        = &'collection T
    where
        Self: 'collection;

    fn index(&self, index: usize) -> Option<Self::RefItem<'_>> {
        self.get(index)
    }

    type Iter<'collection>
        = slice::Iter<'collection, T>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }

    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<T> CollectionMut for Vec<T> {
    type RefItemMut<'collection>
        = &'collection mut T
    where
        Self: 'collection;

    fn index_mut(&mut self, index: usize) -> Option<Self::RefItemMut<'_>> {
        self.get_mut(index)
    }

    type IterMut<'collection>
        = slice::IterMut<'collection, T>
    where
        Self: 'collection;

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        <&mut Self as IntoIterator>::into_iter(self)
    }
}

impl<T> CollectionAlloc for Vec<T> {
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }

    fn reserve(&mut self, additional: usize) {
        Self::reserve(self, additional);
    }
}

impl<T, const N: usize> Collection for [T; N] {
    type Item = T;

    type RefItem<'collection>
        = &'collection T
    where
        Self: 'collection;

    fn index(&self, index: usize) -> Option<Self::RefItem<'_>> {
        self.get(index)
    }

    type Iter<'collection>
        = slice::Iter<'collection, T>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }

    type IntoIter = array::IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<T, const N: usize> CollectionMut for [T; N] {
    type RefItemMut<'collection>
        = &'collection mut T
    where
        Self: 'collection;

    fn index_mut(&mut self, index: usize) -> Option<Self::RefItemMut<'_>> {
        self.get_mut(index)
    }

    type IterMut<'collection>
        = slice::IterMut<'collection, T>
    where
        Self: 'collection;

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        <&mut Self as IntoIterator>::into_iter(self)
    }
}

impl<'a, T: Copy> Collection for &'a [T] {
    type Item = T;

    type RefItem<'collection>
        = &'collection T
    where
        Self: 'collection;

    fn index(&self, index: usize) -> Option<Self::RefItem<'_>> {
        self.get(index)
    }

    type Iter<'collection>
        = slice::Iter<'collection, T>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <Self as IntoIterator>::into_iter(self)
    }

    type IntoIter = Copied<slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self).copied()
    }
}

impl<'a, T: Copy> Collection for &'a mut [T] {
    type Item = T;

    type RefItem<'collection>
        = &'collection T
    where
        Self: 'collection;

    fn index(&self, index: usize) -> Option<Self::RefItem<'_>> {
        self.get(index)
    }

    type Iter<'collection>
        = slice::Iter<'collection, T>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T] as IntoIterator>::into_iter(self)
    }

    type IntoIter = Copied<slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        <&[T] as IntoIterator>::into_iter(self).copied()
    }
}

impl<T: Copy> CollectionMut for &mut [T] {
    type RefItemMut<'collection>
        = &'collection mut T
    where
        Self: 'collection;

    fn index_mut(&mut self, index: usize) -> Option<Self::RefItemMut<'_>> {
        self.get_mut(index)
    }

    type IterMut<'collection>
        = slice::IterMut<'collection, T>
    where
        Self: 'collection;

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        <&mut [T] as IntoIterator>::into_iter(self)
    }
}

impl<T> Collection for Box<[T]> {
    type Item = T;

    type RefItem<'collection>
        = &'collection T
    where
        Self: 'collection;

    fn index(&self, index: usize) -> Option<Self::RefItem<'_>> {
        self.get(index)
    }

    type Iter<'collection>
        = slice::Iter<'collection, T>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }

    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

/// An iterator over copied items `T` in a slice borrowed from `U`.
pub struct BorrowSliceIntoIter<T: Copy, U: Borrow<[T]>> {
    /// The slice is borrowed from this field.
    data: U,
    /// The current position of this iterator.
    index: usize,
    /// The item type.
    _ty: PhantomData<T>,
}

impl<T: Copy, U: Borrow<[T]>> From<U> for BorrowSliceIntoIter<T, U> {
    fn from(data: U) -> Self {
        Self {
            data,
            index: 0,
            _ty: PhantomData,
        }
    }
}

impl<T: Copy, U: Borrow<[T]>> Iterator for BorrowSliceIntoIter<T, U> {
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

impl<T: Copy, U: Borrow<[T]>> ExactSizeIterator for BorrowSliceIntoIter<T, U> {}

impl<T: Copy> Collection for Rc<[T]> {
    type Item = T;

    type RefItem<'collection>
        = &'collection T
    where
        Self: 'collection;

    fn index(&self, index: usize) -> Option<Self::RefItem<'_>> {
        self.get(index)
    }

    type Iter<'collection>
        = slice::Iter<'collection, T>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T] as IntoIterator>::into_iter(self)
    }

    type IntoIter = BorrowSliceIntoIter<T, Self>;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

impl<T: Copy> Collection for Arc<[T]> {
    type Item = T;

    type RefItem<'collection>
        = &'collection T
    where
        Self: 'collection;

    fn index(&self, index: usize) -> Option<Self::RefItem<'_>> {
        self.get(index)
    }

    type Iter<'collection>
        = slice::Iter<'collection, T>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T] as IntoIterator>::into_iter(self)
    }

    type IntoIter = BorrowSliceIntoIter<T, Self>;

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
        let mut iter = BorrowSliceIntoIter::from(input);
        assert_eq!(iter.size_hint(), (4, Some(4)));
        let _ = iter.next();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        let _ = iter.nth(2);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        let _ = iter.next();
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }
}
