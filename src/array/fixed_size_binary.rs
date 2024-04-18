//! Array with fixed-size binary values.

use crate::{
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{BufferType, VecBuffer},
    validity::{Nullability, Validity},
    Index, Length,
};

use super::{Array, FixedSizeListArray, FixedSizePrimitiveArray};

/// Array with fixed-size binary elements.
// to support `arrow-rs` interop we can't use
// FixedSizePrimitiveArray<[u8; N], NULLABLE, Buffer>
pub struct FixedSizeBinaryArray<
    const N: usize,
    const NULLABLE: bool = false,
    Buffer: BufferType = VecBuffer,
>(pub(crate) FixedSizeListArray<N, FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, Buffer>)
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>;

impl<const N: usize, const NULLABLE: bool, Buffer: BufferType>
    FixedSizeBinaryArray<N, NULLABLE, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    FixedSizeBinaryArray<N, NULLABLE, Buffer>: Index + Length,
{
    /// Returns an iterator over items in this [`FixedSizeListArray`].
    pub fn iter(&self) -> FixedSizeBinaryIter<'_, N, NULLABLE, Buffer> {
        <&Self as IntoIterator>::into_iter(self)
    }
}

impl<const N: usize, const NULLABLE: bool, Buffer: BufferType> Array
    for FixedSizeBinaryArray<N, NULLABLE, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    [u8; N]: Nullability<NULLABLE>,
{
    type Item = <[u8; N] as Nullability<NULLABLE>>::Item;
}

impl<const N: usize, Buffer: BufferType> BitmapRef for FixedSizeBinaryArray<N, true, Buffer> {
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<const N: usize, Buffer: BufferType> BitmapRefMut for FixedSizeBinaryArray<N, true, Buffer> {
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<const N: usize, const NULLABLE: bool, Buffer: BufferType> Default
    for FixedSizeBinaryArray<N, NULLABLE, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, Buffer>: Default,
{
    fn default() -> Self {
        Self(FixedSizeListArray::default())
    }
}

impl<const N: usize, Buffer: BufferType> Extend<[u8; N]> for FixedSizeBinaryArray<N, false, Buffer>
where
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, false, Buffer>, false, Buffer>:
        Extend<[u8; N]>,
{
    fn extend<I: IntoIterator<Item = [u8; N]>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<const N: usize, Buffer: BufferType> Extend<Option<[u8; N]>>
    for FixedSizeBinaryArray<N, true, Buffer>
where
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, false, Buffer>, true, Buffer>:
        Extend<Option<[u8; N]>>,
{
    fn extend<I: IntoIterator<Item = Option<[u8; N]>>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<const N: usize, Buffer: BufferType> From<FixedSizeBinaryArray<N, false, Buffer>>
    for FixedSizeBinaryArray<N, true, Buffer>
where
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, false, Buffer>, true, Buffer>:
        From<FixedSizeListArray<N, FixedSizePrimitiveArray<u8, false, Buffer>, false, Buffer>>,
{
    fn from(value: FixedSizeBinaryArray<N, false, Buffer>) -> Self {
        Self(value.0.into())
    }
}

impl<const N: usize, Buffer: BufferType> FromIterator<[u8; N]>
    for FixedSizeBinaryArray<N, false, Buffer>
where
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, false, Buffer>, false, Buffer>:
        FromIterator<[u8; N]>,
{
    fn from_iter<I: IntoIterator<Item = [u8; N]>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<const N: usize, Buffer: BufferType> FromIterator<Option<[u8; N]>>
    for FixedSizeBinaryArray<N, true, Buffer>
where
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, false, Buffer>, true, Buffer>:
        FromIterator<Option<[u8; N]>>,
{
    fn from_iter<I: IntoIterator<Item = Option<[u8; N]>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<const N: usize, const NULLABLE: bool, Buffer: BufferType> Index
    for FixedSizeBinaryArray<N, NULLABLE, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, Buffer>: Index,
{
    type Item<'a> = <FixedSizeListArray<N, FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, Buffer> as Index>::Item<'a>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.0.index_unchecked(index)
    }
}

/// An iterator over fixed-size lists in a [`FixedSizeBinaryArray`].
pub struct FixedSizeBinaryIter<'a, const N: usize, const NULLABLE: bool, Buffer: BufferType>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
{
    /// Reference to the array.
    array: &'a FixedSizeBinaryArray<N, NULLABLE, Buffer>,
    /// Current index.
    index: usize,
}

impl<'a, const N: usize, const NULLABLE: bool, Buffer: BufferType> Iterator
    for FixedSizeBinaryIter<'a, N, NULLABLE, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    FixedSizeBinaryArray<N, NULLABLE, Buffer>: Length + Index,
{
    type Item = <FixedSizeBinaryArray<N, NULLABLE, Buffer> as Index>::Item<'a>;

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

impl<'a, const N: usize, const NULLABLE: bool, Buffer: BufferType> IntoIterator
    for &'a FixedSizeBinaryArray<N, NULLABLE, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    FixedSizeBinaryArray<N, NULLABLE, Buffer>: Index + Length,
{
    type Item = <FixedSizeBinaryArray<N, NULLABLE, Buffer> as Index>::Item<'a>;
    type IntoIter = FixedSizeBinaryIter<'a, N, NULLABLE, Buffer>;

    fn into_iter(self) -> Self::IntoIter {
        FixedSizeBinaryIter {
            array: self,
            index: 0,
        }
    }
}

impl<const N: usize, const NULLABLE: bool, Buffer: BufferType> Length
    for FixedSizeBinaryArray<N, NULLABLE, Buffer>
where
    FixedSizePrimitiveArray<u8, false, Buffer>: Validity<NULLABLE>,
    FixedSizeListArray<N, FixedSizePrimitiveArray<u8, false, Buffer>, NULLABLE, Buffer>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<const N: usize, Buffer: BufferType> ValidityBitmap for FixedSizeBinaryArray<N, true, Buffer> {}

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
            .collect::<FixedSizeBinaryArray<2, true>>();
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
            .collect::<FixedSizeBinaryArray<2, true>>();
        assert_eq!(array_nullable.index(0), Some(Some([&1, &2])));
        assert_eq!(array_nullable.index(1), Some(None));
        assert_eq!(array_nullable.index(2), None);
    }

    #[test]
    fn into_iter() {
        let input = [[1_u8, 2], [3, 4]];
        let array = input.into_iter().collect::<FixedSizeBinaryArray<2>>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), [[&1, &2], [&3, &4]]);

        let input_nullable = [Some([1_u8, 2]), None];
        let array_nullable = input_nullable
            .into_iter()
            .collect::<FixedSizeBinaryArray<2, true>>();
        assert_eq!(
            array_nullable.into_iter().collect::<Vec<_>>(),
            [Some([&1, &2]), None]
        );
    }
}
