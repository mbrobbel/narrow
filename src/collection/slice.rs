use core::{borrow::Borrow, iter::Map, marker::PhantomData, slice};

use crate::collection::{Collection, owned::IntoOwned, view::AsView};

impl<T: for<'any> AsView<'any>> Collection for &[T] {
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
        <Self as IntoIterator>::into_iter(self).map(AsView::as_view)
    }

    type IntoIter = SliceIntoIter<Self, T>;

    fn into_iter_owned(self) -> Self::IntoIter {
        self.into()
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
                self.index = self.index.strict_add(1);
            })
            .map(|item| item.as_view().into_owned())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.slice.borrow().len().strict_sub(self.index);
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

#[cfg(test)]
mod tests {
    use super::*;

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
