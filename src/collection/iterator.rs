//! Collection iterators

/// Iterator trait for [`Collection`] that support returning items that borrow
/// from the iterator.
pub trait CollectionIterator {
    /// The item returned by this iterator.
    type Item<'collection>
    where
        Self: 'collection;

    /// Return the next item in this iterator.
    fn next(&mut self) -> Option<Self::Item<'_>>;
}

impl<I: Iterator> CollectionIterator for I {
    type Item<'collection>
        = <I as Iterator>::Item
    where
        Self: 'collection;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        <Self as Iterator>::next(self)
    }
}
