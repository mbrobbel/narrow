//! Array with fixed-size binary values.

use crate::{
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{Buffer, BufferType, VecBuffer},
    nullability::{NonNullable, Nullability, Nullable},
    Index, Length,
};

use super::{Array, FixedSizeListArray, FixedSizePrimitiveArray};

/// Array with fixed-size binary elements.
// to support `arrow-rs` interop we can't use
// FixedSizePrimitiveArray<[u8; N], NULLABLE, Buffer>
pub struct FixedSizeBinaryArray<
    const N: usize,
    Nullable: Nullability = NonNullable,
    Buffer: BufferType = VecBuffer,
>(
    #[rustfmt::skip]
    pub(crate)
        FixedSizeListArray<N, FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, Buffer>,
);

impl<const N: usize, Nullable: Nullability, Buffer: BufferType>
    FixedSizeBinaryArray<N, Nullable, Buffer>
where
    FixedSizeBinaryArray<N, Nullable, Buffer>: Index + Length,
{
    /// Returns an iterator over items in this [`FixedSizeListArray`].
    pub fn iter(&self) -> FixedSizeBinaryIter<'_, N, Nullable, Buffer> {
        <&Self as IntoIterator>::into_iter(self)
    }
}

impl<const N: usize, Nullable: Nullability, Buffer: BufferType> Array
    for FixedSizeBinaryArray<N, Nullable, Buffer>
{
    type Item = Nullable::Item<[u8; N]>;
}

impl<const N: usize, Buffer: BufferType> BitmapRef for FixedSizeBinaryArray<N, Nullable, Buffer> {
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<const N: usize, Buffer: BufferType> BitmapRefMut
    for FixedSizeBinaryArray<N, Nullable, Buffer>
{
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<const N: usize, Nullable: Nullability, Buffer: BufferType> Clone
    for FixedSizeBinaryArray<N, Nullable, Buffer>
where
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, Buffer>:
        Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<const N: usize, Nullable: Nullability, Buffer: BufferType> Default
    for FixedSizeBinaryArray<N, Nullable, Buffer>
where
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, Buffer>:
        Default,
{
    fn default() -> Self {
        Self(FixedSizeListArray::default())
    }
}

impl<T, const N: usize, Buffer: BufferType> Extend<T>
    for FixedSizeBinaryArray<N, NonNullable, Buffer>
where
    T: Into<[u8; N]>,
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, NonNullable, Buffer>, NonNullable, Buffer>:
        Extend<[u8; N]>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().map(Into::into));
    }
}

impl<T, const N: usize, Buffer: BufferType> Extend<Option<T>>
    for FixedSizeBinaryArray<N, Nullable, Buffer>
where
    T: Into<[u8; N]>,
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, Buffer>:
        Extend<Option<[u8; N]>>,
{
    fn extend<I: IntoIterator<Item = Option<T>>>(&mut self, iter: I) {
        self.0
            .extend(iter.into_iter().map(|opt| opt.map(Into::into)));
    }
}

impl<const N: usize, Buffer: BufferType> From<FixedSizeBinaryArray<N, NonNullable, Buffer>>
    for FixedSizeBinaryArray<N, Nullable, Buffer>
where
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, Buffer>: From<
        FixedSizeListArray<
            N,
            FixedSizePrimitiveArray<u8, NonNullable, Buffer>,
            NonNullable,
            Buffer,
        >,
    >,
{
    fn from(value: FixedSizeBinaryArray<N, NonNullable, Buffer>) -> Self {
        Self(value.0.into())
    }
}

impl<T, const N: usize, Buffer: BufferType> FromIterator<T>
    for FixedSizeBinaryArray<N, NonNullable, Buffer>
where
    T: Into<[u8; N]>,
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, NonNullable, Buffer>, NonNullable, Buffer>:
        FromIterator<[u8; N]>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

impl<T, const N: usize, Buffer: BufferType> FromIterator<Option<T>>
    for FixedSizeBinaryArray<N, Nullable, Buffer>
where
    T: Into<[u8; N]>,
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, Buffer>:
        FromIterator<Option<[u8; N]>>,
{
    fn from_iter<I: IntoIterator<Item = Option<T>>>(iter: I) -> Self {
        Self(iter.into_iter().map(|opt| opt.map(Into::into)).collect())
    }
}

impl<const N: usize, Nullable: Nullability, Buffer: BufferType> Index
    for FixedSizeBinaryArray<N, Nullable, Buffer>
where
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, Buffer>:
        Index,
{
    type Item<'a> = <FixedSizeListArray<N, FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, Buffer> as Index>::Item<'a>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.0.index_unchecked(index)
    }
}

/// An iterator over fixed-size lists in a [`FixedSizeBinaryArray`].
pub struct FixedSizeBinaryIntoIter<const N: usize, Nullable: Nullability, Buffer: BufferType> {
    /// Reference to the array.
    array: FixedSizeBinaryArray<N, Nullable, Buffer>,
    /// Current index.
    index: usize,
}

impl<const N: usize, Buffer: BufferType> Iterator
    for FixedSizeBinaryIntoIter<N, NonNullable, Buffer>
{
    type Item = [u8; N];

    fn next(&mut self) -> Option<Self::Item> {
        (self.index < self.array.len()).then(|| {
            let item = self.array.0 .0 .0.as_slice()[self.index * N..self.index * N + N]
                .try_into()
                .expect("out of bounds");
            self.index += 1;
            item
        })
    }
}

impl<const N: usize, Buffer: BufferType> Iterator for FixedSizeBinaryIntoIter<N, Nullable, Buffer> {
    type Item = Option<[u8; N]>;

    fn next(&mut self) -> Option<Self::Item> {
        (self.index < self.array.len()).then(|| {
            // Safety:
            // - bound checked above
            let item = unsafe { self.array.0 .0.is_valid_unchecked(self.index) }.then(|| {
                self.array.0 .0.data.0.as_slice()[self.index * N..self.index * N + N]
                    .try_into()
                    .expect("out of bounds")
            });
            self.index += 1;
            item
        })
    }
}

/// An iterator over fixed-size lists in a [`FixedSizeBinaryArray`].
pub struct FixedSizeBinaryIter<'a, const N: usize, Nullable: Nullability, Buffer: BufferType> {
    /// Reference to the array.
    array: &'a FixedSizeBinaryArray<N, Nullable, Buffer>,
    /// Current index.
    index: usize,
}

impl<'a, const N: usize, Nullable: Nullability, Buffer: BufferType> Iterator
    for FixedSizeBinaryIter<'a, N, Nullable, Buffer>
where
    FixedSizeBinaryArray<N, Nullable, Buffer>: Length + Index,
{
    type Item = <FixedSizeBinaryArray<N, Nullable, Buffer> as Index>::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.array
            .index(self.index)
            .into_iter()
            .inspect(|_| {
                self.index += 1;
            })
            .next()
    }
}

impl<const N: usize, Nullable: Nullability, Buffer: BufferType> IntoIterator
    for FixedSizeBinaryArray<N, Nullable, Buffer>
where
    FixedSizeBinaryIntoIter<N, Nullable, Buffer>: Iterator,
{
    type Item = <FixedSizeBinaryIntoIter<N, Nullable, Buffer> as Iterator>::Item;
    type IntoIter = FixedSizeBinaryIntoIter<N, Nullable, Buffer>;

    fn into_iter(self) -> Self::IntoIter {
        FixedSizeBinaryIntoIter {
            array: self,
            index: 0,
        }
    }
}

impl<'a, const N: usize, Nullable: Nullability, Buffer: BufferType> IntoIterator
    for &'a FixedSizeBinaryArray<N, Nullable, Buffer>
where
    FixedSizeBinaryArray<N, Nullable, Buffer>: Index + Length,
{
    type Item = <FixedSizeBinaryArray<N, Nullable, Buffer> as Index>::Item<'a>;
    type IntoIter = FixedSizeBinaryIter<'a, N, Nullable, Buffer>;

    fn into_iter(self) -> Self::IntoIter {
        FixedSizeBinaryIter {
            array: self,
            index: 0,
        }
    }
}

impl<const N: usize, Nullable: Nullability, Buffer: BufferType> Length
    for FixedSizeBinaryArray<N, Nullable, Buffer>
where
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, NonNullable, Buffer>, Nullable, Buffer>:
        Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<const N: usize, Buffer: BufferType> ValidityBitmap
    for FixedSizeBinaryArray<N, Nullable, Buffer>
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        let input = [[1_u8, 2], [3, 4]];
        let array = input.into_iter().collect::<FixedSizeBinaryArray<2>>();
        assert_eq!(array.len(), 2);
        assert_eq!(array.0 .0.len(), 4);

        let input_nullable = [Some([1_u8, 2]), None];
        let array_nullable = input_nullable
            .into_iter()
            .collect::<FixedSizeBinaryArray<2, Nullable>>();
        assert_eq!(array_nullable.len(), 2);
        assert_eq!(array_nullable.0 .0.data.len(), 4);
        assert_eq!(array_nullable.0 .0.validity.len(), 2);
    }

    #[test]
    fn index() {
        let input = [[1_u8, 2], [3, 4]];
        let array = input.into_iter().collect::<FixedSizeBinaryArray<2>>();
        assert_eq!(array.index(0), Some([&1, &2]));
        assert_eq!(array.index(1), Some([&3, &4]));

        let input_nullable = [Some([1_u8, 2]), None];
        let array_nullable = input_nullable
            .into_iter()
            .collect::<FixedSizeBinaryArray<2, Nullable>>();
        assert_eq!(array_nullable.index(0), Some(Some([&1, &2])));
        assert_eq!(array_nullable.index(1), Some(None));
        assert_eq!(array_nullable.index(2), None);
    }

    #[test]
    fn into_iter() {
        let input = [[1_u8, 2], [3, 4]];
        let array = input.into_iter().collect::<FixedSizeBinaryArray<2>>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), [[1, 2], [3, 4]]);

        let input_nullable = [Some([1_u8, 2]), None];
        let array_nullable = input_nullable
            .into_iter()
            .collect::<FixedSizeBinaryArray<2, Nullable>>();
        assert_eq!(
            array_nullable.into_iter().collect::<Vec<_>>(),
            [Some([1, 2]), None]
        );
    }
}
