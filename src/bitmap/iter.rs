//! Bitmap iteration.

use std::{
    borrow::Borrow,
    iter::{Skip, Take},
    slice,
};

/// An iterator over the bits in a Bitmap.
///
/// This iterator returns boolean values that represent the bits stored in a
/// Bitmap.
pub type BitmapIter<'a> = Take<Skip<BitUnpacked<slice::Iter<'a, u8>, &'a u8>>>;

/// An iterator over the bits in a Bitmap. Consumes the Bitmap.
pub type BitmapIntoIter<I> = Take<Skip<BitUnpacked<I, u8>>>;

/// An iterator that packs boolean values as bits in bytes using
/// least-significant bit (LSB) numbering.
///
/// Wraps around an iterator (`I`) over items (`T`) that can be borrowed as
/// boolean values.
pub struct BitPacked<I, T>
where
    I: Iterator<Item = T>,
    T: Borrow<bool>,
{
    iter: I,
}

impl<I, T> Iterator for BitPacked<I, T>
where
    I: Iterator<Item = T>,
    T: Borrow<bool>,
{
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Get the next item from the inner iterator or return None if the inner
        // iterator is finished.
        self.iter.next().map(|next| {
            // Set the least significant bit based on the first boolean value.
            let mut byte = u8::from(*next.borrow());
            for bit_position in 1u8..8 {
                // If the inner iterator has more boolean values and they are set
                // (`true`), set the corresponding bit in the output byte.
                if let Some(x) = self.iter.next() {
                    if *x.borrow() {
                        byte |= 1 << bit_position;
                    }
                }
            }
            byte
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();

        #[inline]
        const fn bytes_for_bits(bits: usize) -> usize {
            bits.saturating_add(7) / 8
        }

        // One item is returned per 8 items in the inner iterator.
        (bytes_for_bits(lower), upper.map(bytes_for_bits))
    }

    // todo(mb): advance_by, nth
}

// If the inner iterator is ExactSizeIterator, the bounds reported by
// the size hint of this iterator are exact.
impl<I, T> ExactSizeIterator for BitPacked<I, T>
where
    I: ExactSizeIterator<Item = T>,
    T: Borrow<bool>,
{
}

/// An [Iterator] extension trait for [BitPacked].
pub trait BitPackedExt<T>
where
    Self: Iterator<Item = T>,
    T: Borrow<bool>,
{
    /// Packs the items in this iterator that can be borrowed as boolean values
    /// as bits in bytes using least-significant bit (LSB) numbering.
    fn bit_packed(self) -> BitPacked<Self, T>
    where
        Self: Sized,
    {
        BitPacked { iter: self }
    }
}

impl<I, T> BitPackedExt<T> for I
where
    I: Iterator<Item = T>,
    T: Borrow<bool>,
{
}

/// An iterator that unpacks boolean values from an iterator (`I`) over items
/// (`T`) that can be borrowed as bytes, by interpreting the bits of these bytes
/// with least-significant bit (LSB) numbering as boolean values i.e. `1` maps
/// to `true` and `0` maps to `false`.
pub struct BitUnpacked<I, T>
where
    I: Iterator<Item = T>,
    T: Borrow<u8>,
{
    iter: I,
    byte: Option<u8>,
    mask: u8,
}

impl<I, T> Iterator for BitUnpacked<I, T>
where
    I: Iterator<Item = T>,
    T: Borrow<u8> + Copy,
{
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Check if we need to fetch the next byte from the inner iterator.
        if self.mask == 0x01 {
            self.byte = self.iter.next().map(|item| *item.borrow());
        }

        // If we have a byte there are still boolean values to yield.
        self.byte.map(|byte| {
            let next = (byte & self.mask) != 0;
            self.mask = self.mask.rotate_left(1);
            next
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();

        // 8 items are returned per one item in the inner iterator.
        (
            lower.saturating_mul(8),
            upper.and_then(|upper| upper.checked_mul(8)),
        )
    }

    // todo(mb): advance_by, nth
}

/// An [Iterator] extension trait for [BitUnpacked].
pub trait BitUnpackedExt<T>
where
    Self: Iterator<Item = T>,
    T: Borrow<u8>,
{
    fn bit_unpacked(self) -> BitUnpacked<Self, T>
    where
        Self: Sized,
    {
        BitUnpacked {
            iter: self,
            byte: None,
            mask: 0x01,
        }
    }
}

impl<I, T> BitUnpackedExt<T> for I
where
    I: Iterator<Item = T>,
    T: Borrow<u8>,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack() {
        let input = [false, true, false, true, false, true];
        assert_eq!(input.iter().bit_packed().next(), Some(0x2a));
        let input = [false, true, false, true, false, true, false, true];
        assert_eq!(input.iter().bit_packed().next(), Some(0xaa));
        let input = [true; 16];
        assert_eq!(input.iter().bit_packed().collect::<Vec<u8>>(), [0xff, 0xff]);
    }

    #[test]
    fn unpack() {
        let input = [u8::MAX, 1];
        assert_eq!(
            input.iter().bit_unpacked().collect::<Vec<_>>(),
            vec![
                true, true, true, true, true, true, true, true, true, false, false, false, false,
                false, false, false
            ]
        );
    }

    #[test]
    fn unpack_size_hint() {
        let input = [u8::MAX, 1, 2, 3];
        assert_eq!(
            input.iter().bit_unpacked().size_hint(),
            (input.len() * 8, Some(input.len() * 8))
        );
    }

    #[test]
    fn pack_size_hint() {
        assert_eq!(
            (usize::MAX / 8, None),
            (0..).into_iter().map(|_| true).bit_packed().size_hint()
        );
        assert_eq!(
            (usize::MAX / 8, None),
            (0..=usize::MAX)
                .into_iter()
                .map(|_| true)
                .bit_packed()
                .size_hint()
        );
        assert_eq!(
            (usize::MAX / 8, Some(usize::MAX / 8)),
            (0..usize::MAX)
                .into_iter()
                .map(|_| true)
                .bit_packed()
                .size_hint()
        );
        assert_eq!(
            (1, Some(1)),
            (0..3).into_iter().map(|_| true).bit_packed().size_hint()
        );
    }

    #[test]
    fn round_trip() {
        let input = [false, true, false, true, false, true];
        assert_eq!(
            input
                .iter()
                .bit_packed()
                .bit_unpacked()
                .take(input.len())
                .collect::<Vec<bool>>(),
            input
        );
        let input = [true, true, false, true, false, true, true, true];
        assert_eq!(
            input
                .iter()
                .bit_packed()
                .bit_unpacked()
                .collect::<Vec<bool>>(),
            input
        );
    }
}
