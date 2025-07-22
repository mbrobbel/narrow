//! Arrow memory layouts.

use crate::{
    bitmap::Bitmap,
    buffer::{BufferType, VecBuffer},
    collection::{Collection, Item},
    length::Length,
    validity::Validity,
    // nullability::{Nullability, Nullable},
};

/// A physical memory layout.
pub trait Layout: Item {
    /// The storage used for this layout, for nullable and non-nullable using the given buffer type.
    type Storage<Buffer: BufferType>: Collection<Item = Self>;
}

// pub mod fixed_size_primitive;

impl Layout for bool {
    type Storage<Buffer: BufferType> = Bitmap<Buffer>;
}

impl Layout for u8 {
    type Storage<Buffer: BufferType> = Buffer::Buffer<u8>;
}

impl Layout for i16 {
    type Storage<Buffer: BufferType> = Buffer::Buffer<i16>;
}

impl Layout for i32 {
    type Storage<Buffer: BufferType> = Buffer::Buffer<i32>;
}

impl<T: Layout> Layout for Option<T> {
    type Storage<Buffer: BufferType> = Validity<T::Storage<Buffer>, Buffer>;
}

/// An arrow arry for type T.
#[derive(Default, Debug)]
pub struct Array<T: Layout, Buffer: BufferType = VecBuffer>(T::Storage<Buffer>);

impl<T: Layout, Buffer: BufferType> Length for Array<T, Buffer> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Layout<Storage<Buffer>: FromIterator<T>>, Buffer: BufferType> FromIterator<T>
    for Array<T, Buffer>
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: Layout<Storage<Buffer>: Extend<T>>, Buffer: BufferType> Extend<T> for Array<T, Buffer> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<T: Layout, Buffer: BufferType> Collection for Array<T, Buffer> {
    type Item = <T::Storage<Buffer> as Collection>::Item;

    fn index(&self, index: usize) -> Option<<Self::Item as crate::collection::Item>::Ref<'_>> {
        self.0.index(index)
    }

    type Iter<'collection>
        = <<T as Layout>::Storage<Buffer> as Collection>::Iter<'collection>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter()
    }

    type IntoIter = <<T as Layout>::Storage<Buffer> as Collection>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::BoxBuffer;

    use super::*;

    #[test]
    fn a() {
        let non_nullable = Collection::into_iter([1, 2, 3, 4]).collect::<Array<_, BoxBuffer>>();
        assert_eq!(non_nullable.index(0), Some(1));

        let mut non_nullable = Collection::into_iter([1, 2, 3, 4]).collect::<Array<_>>();
        assert_eq!(non_nullable.index(0), Some(1));
        non_nullable.extend([5]);
        assert_eq!(non_nullable.index(4), Some(5));

        let mut non_nullable =
            Collection::into_iter([true, false, true, true]).collect::<Array<_>>();
        assert_eq!(non_nullable.index(0), Some(true));
        non_nullable.extend([false]);
        assert_eq!(non_nullable.index(4), Some(false));

        let mut non_nullable = Collection::into_iter([Some(true), Some(false), None, Some(true)])
            .collect::<Array<_>>();
        assert_eq!(non_nullable.index(0), Some(Some(true)));
        non_nullable.extend([Some(false)]);
        assert_eq!(non_nullable.index(4), Some(Some(false)));
    }
}
