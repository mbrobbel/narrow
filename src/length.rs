/// The length of collections.
pub trait Length {
    /// Returns the number of elements in the collection, also referred to as its length.
    fn len(&self) -> usize;

    /// Returns `true` if there are no elements in the collection.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
