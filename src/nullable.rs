//! Nullable data.

use std::iter::{Map, Zip};

use crate::{
    bitmap::{
        iter::{BitmapIntoIter, BitmapIter},
        Bitmap,
    },
    buffer::{Buffer, BufferExtend},
    DataBuffer, Length, Null, Primitive, ValidityBitmap,
};

/// Wrapper for nullable data.
///
/// Store data with a validity [Bitmap] that uses a single bit per value in `T`
/// that indicates the nullness or non-nullness of that value.
#[derive(Debug)]
pub struct Nullable<T, U = Vec<u8>>
where
    U: Buffer<u8>,
{
    data: T,
    // TODO: wrap Bitmap in Option to handle external data for nullable types that don't have a
    // validity buffer allocated. None indicates all the value in T are valid.
    validity: Bitmap<U>,
}

impl<T, U, V> FromIterator<Option<V>> for Nullable<T, U>
where
    T: Default + BufferExtend<V>,
    U: Default + BufferExtend<u8>,
    V: Primitive,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<V>>,
    {
        let (validity, data) = iter
            .into_iter()
            .map(|opt| (opt.is_some(), opt.unwrap_or_default()))
            .unzip();

        Self { data, validity }
    }
}

impl<T, U> IntoIterator for Nullable<T, U>
where
    U: IntoIterator<Item = u8>,
    T: IntoIterator,
    U: Buffer<u8>,
{
    type IntoIter = Map<
        Zip<BitmapIntoIter<<U as IntoIterator>::IntoIter>, <T as IntoIterator>::IntoIter>,
        fn((bool, <T as IntoIterator>::Item)) -> Self::Item,
    >;
    type Item = Option<<T as IntoIterator>::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.validity
            .into_iter()
            .zip(self.data.into_iter())
            .map(|(validity, value)| validity.then_some(value))
    }
}

impl<'a, T, U> IntoIterator for &'a Nullable<T, U>
where
    &'a T: IntoIterator,
    U: Buffer<u8>,
{
    type IntoIter = Map<
        Zip<BitmapIter<'a>, <&'a T as IntoIterator>::IntoIter>,
        fn((bool, <&'a T as IntoIterator>::Item)) -> Self::Item,
    >;
    type Item = Option<<&'a T as IntoIterator>::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.validity
            .into_iter()
            .zip(self.data.into_iter())
            .map(|(validity, value)| validity.then_some(value))
    }
}

impl<T, U, V> DataBuffer<V> for Nullable<T, U>
where
    T: Buffer<V>,
    U: Buffer<u8>,
    V: Primitive,
{
    type Buffer = T;

    fn data_buffer(&self) -> &Self::Buffer {
        &self.data
    }
}

impl<T, U> ValidityBitmap<U> for Nullable<T, U>
where
    U: Buffer<u8>,
{
    fn validity_bitmap(&self) -> &Bitmap<U> {
        &self.validity
    }
}

impl<T, U> Length for Nullable<T, U>
where
    U: Buffer<u8>,
{
    fn len(&self) -> usize {
        self.validity.len()
    }
}

impl<T, U> Null for Nullable<T, U>
where
    U: Buffer<u8>,
{
    unsafe fn is_valid_unchecked(&self, index: usize) -> bool {
        self.validity.is_valid_unchecked(index)
    }

    fn is_null(&self, index: usize) -> Option<bool> {
        self.validity.is_null(index)
    }

    unsafe fn is_null_unchecked(&self, index: usize) -> bool {
        self.validity.is_null_unchecked(index)
    }

    fn null_count(&self) -> usize {
        self.validity.null_count()
    }

    fn is_valid(&self, index: usize) -> Option<bool> {
        self.validity.is_valid(index)
    }

    fn valid_count(&self) -> usize {
        self.validity.valid_count()
    }

    fn any_null(&self) -> bool {
        self.validity.any_null()
    }

    fn all_null(&self) -> bool {
        self.validity.all_null()
    }

    fn any_valid(&self) -> bool {
        self.validity.any_valid()
    }

    fn all_valid(&self) -> bool {
        self.validity.all_valid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        let input = [Some(1u32), Some(2), Some(3), Some(4), None, Some(42)];
        let nullable = input.into_iter().collect::<Nullable<Vec<_>>>();
        assert_eq!(nullable.data_buffer(), &[1, 2, 3, 4, u32::default(), 42]);
        assert_eq!(nullable.validity.data_buffer(), &[0b00101111u8]);
    }

    #[test]
    fn into_iter() {
        let input = [Some(1u32), Some(2), Some(3), Some(4), None, Some(42)];
        let nullable = input.into_iter().collect::<Nullable<Vec<_>>>();
        let output = nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());
    }
}
