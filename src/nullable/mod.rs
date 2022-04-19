//! Nullable data.

use crate::{
    bitmap::Bitmap,
    buffer::{Buffer, BufferExtend},
    DataBuffer, Length, Null, Primitive,
};

/// Wrapper for nullable data.
///
/// Store data with a validity [Bitmap] that uses a single bit per value in `T`
/// that indicates the nullness or non-nullness of that value.
pub struct Nullable<T, U>
where
    U: Buffer<u8>,
{
    data: T,
    validity: Bitmap<U>,
}

impl<T, U, V> FromIterator<Option<V>> for Nullable<T, U>
where
    U: Default + BufferExtend<u8>,
    T: Default + BufferExtend<V>,
    V: Primitive,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<V>>,
    {
        let (data, validity) = iter
            .into_iter()
            .map(|opt| (opt.unwrap_or_default(), opt.is_some()))
            .unzip();

        Self { data, validity }
    }
}

impl<T, U, V> DataBuffer<V> for Nullable<T, U>
where
    V: Primitive,
    U: Buffer<u8>,
    T: Buffer<V>,
{
    type Buffer = T;

    fn data_buffer(&self) -> &Self::Buffer {
        &self.data
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        let input = [Some(1u32), Some(2), Some(3), Some(4), None, Some(42)];
        let nullable = input.into_iter().collect::<Nullable<Vec<_>, Vec<_>>>();
        assert_eq!(nullable.data_buffer(), &[1, 2, 3, 4, u32::default(), 42]);
        assert_eq!(nullable.validity.data_buffer(), &[0b00101111u8]);
    }
}
