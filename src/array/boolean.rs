//! Array with boolean values.

use crate::{
    bitmap::Bitmap,
    buffer::{BufferType, VecBuffer},
    validity::Validity,
    Length,
};

use super::Array;

/// Array with boolean values.
///
/// Values are stored using single bits in a [Bitmap].
pub struct BooleanArray<const NULLABLE: bool = false, Buffer: BufferType = VecBuffer>(
    <Bitmap<Buffer> as Validity<NULLABLE>>::Storage<Buffer>,
)
where
    Bitmap<Buffer>: Validity<NULLABLE>;

impl<const NULLABLE: bool, Buffer: BufferType> Array for BooleanArray<NULLABLE, Buffer>
where
    Bitmap<Buffer>: Validity<NULLABLE>,
{
    type Item = bool;
}

impl<const NULLABLE: bool, Buffer: BufferType> Default for BooleanArray<NULLABLE, Buffer>
where
    Bitmap<Buffer>: Validity<NULLABLE>,
    <Bitmap<Buffer> as Validity<NULLABLE>>::Storage<Buffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<U, const NULLABLE: bool, Buffer: BufferType> Extend<U> for BooleanArray<NULLABLE, Buffer>
where
    Bitmap<Buffer>: Validity<NULLABLE>,
    <Bitmap<Buffer> as Validity<NULLABLE>>::Storage<Buffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<const NULLABLE: bool, U, Buffer: BufferType> FromIterator<U> for BooleanArray<NULLABLE, Buffer>
where
    Bitmap<Buffer>: Validity<NULLABLE>,
    <Bitmap<Buffer> as Validity<NULLABLE>>::Storage<Buffer>: FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a, const NULLABLE: bool, Buffer: BufferType> IntoIterator
    for &'a BooleanArray<NULLABLE, Buffer>
where
    Bitmap<Buffer>: Validity<NULLABLE>,
    &'a <Bitmap<Buffer> as Validity<NULLABLE>>::Storage<Buffer>: IntoIterator,
{
    type Item = <&'a <Bitmap<Buffer> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::Item;
    type IntoIter =
        <&'a <Bitmap<Buffer> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const NULLABLE: bool, Buffer: BufferType> IntoIterator for BooleanArray<NULLABLE, Buffer>
where
    Bitmap<Buffer>: Validity<NULLABLE>,
    <Bitmap<Buffer> as Validity<NULLABLE>>::Storage<Buffer>: IntoIterator,
{
    type Item = <<Bitmap<Buffer> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::Item;
    type IntoIter =
        <<Bitmap<Buffer> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const NULLABLE: bool, Buffer: BufferType> Length for BooleanArray<NULLABLE, Buffer>
where
    Bitmap<Buffer>: Validity<NULLABLE>,
    <Bitmap<Buffer> as Validity<NULLABLE>>::Storage<Buffer>: Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
