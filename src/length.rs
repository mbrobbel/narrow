/// The length of a collection.
pub trait Length {
    /// Returns the number of elements in the collection, also referred to as its length.
    fn len(&self) -> usize;

    /// Returns `true` if there are no elements in the collection.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T, const N: usize> Length for [T; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<T> Length for &[T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> Length for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl<T> Length for &T
where
    T: Length,
{
    fn len(&self) -> usize {
        T::len(self)
    }
}

impl Length for &str {
    fn len(&self) -> usize {
        str::len(self)
    }
}
