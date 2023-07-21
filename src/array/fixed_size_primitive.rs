use super::Array;
use crate::{
    buffer::{BufferType, VecBuffer},
    validity::Validity,
    FixedSize,
    Length,
    // Length,
};

/// Array with primitive values.
pub struct FixedSizePrimitiveArray<
    T: FixedSize,
    const NULLABLE: bool = false,
    Buffer: BufferType = VecBuffer,
>(pub <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>)
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>;

impl<T: FixedSize, const NULLABLE: bool, Buffer: BufferType> Array
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
{
    type Item = T;
}

impl<T: FixedSize, const NULLABLE: bool, Buffer: BufferType> Default
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: FixedSize, U, const NULLABLE: bool, Buffer: BufferType> Extend<U>
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<T: FixedSize, const NULLABLE: bool, U, Buffer: BufferType> FromIterator<U>
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a, T: FixedSize, const NULLABLE: bool, Buffer: BufferType> IntoIterator
    for &'a FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    &'a <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: IntoIterator,
{
    type Item = <&'a <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::Item;
    type IntoIter = <&'a <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: FixedSize, const NULLABLE: bool, Buffer: BufferType> IntoIterator
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: IntoIterator,
{
    type Item = <<<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::Item;
    type IntoIter = <<<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: FixedSize, const NULLABLE: bool, Buffer: BufferType> Length
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{Buffer, BufferRef};

    #[test]
    fn from_iter() {
        let input = [1u8, 2, 3, 4];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.0.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(array.0.as_slice(), array.0.as_bytes());

        let input = [[1u8, 2], [3, 4]];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.0.as_slice(), &[[1, 2], [3, 4]]);
        assert_eq!(<_ as Buffer<u8>>::as_bytes(&array.0), &[1, 2, 3, 4]);

        let input = [Some(1u64), None, Some(3), Some(4)];
        let array = input.iter().collect::<FixedSizePrimitiveArray<_, true>>();
        assert_eq!(array.0.buffer_ref().as_slice(), &[1, u64::default(), 3, 4]);
    }

    #[test]
    fn into_iter() {
        let input = [1u8, 2, 3, 4];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), input);

        let input = [[1u8, 2], [3, 4]];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), input);

        let input = [Some(1u64), None, Some(3), Some(4)];
        let array = input.iter().collect::<FixedSizePrimitiveArray<_, true>>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), input);
    }

    #[test]
    fn length() {
        let input = [1u8, 2, 3, 4];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.len(), input.as_slice().len());

        let input = [[1u8, 2], [3, 4]];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.len(), input.as_slice().len());

        let input = [Some(1u64), None, Some(3), Some(4)];
        let array = input.iter().collect::<FixedSizePrimitiveArray<_, true>>();
        assert_eq!(array.len(), input.len());
    }
}
