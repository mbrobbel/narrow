//! An iterator that packs boolean values.

use core::{borrow::Borrow, iter::Fuse};

use super::bytes_for_bits;

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
    /// The inner iterator with boolean values.
    ///
    /// Fused: once the inner iterator returns `None`, items it might yield
    /// afterwards must not be packed at positions following the gap.
    iter: Fuse<I>,
}

impl<I, T> Iterator for BitPacked<I, T>
where
    I: Iterator<Item = T>,
    T: Borrow<bool>,
{
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Fold up to eight items with internal iteration, which inner adapters
        // such as `Chain` and `Inspect` implement more efficiently than
        // repeated calls to `next`.
        let (byte, bits) =
            self.iter
                .by_ref()
                .take(8)
                .fold((0_u8, 0_u32), |(byte, bit_position), next| {
                    (
                        byte | (u8::from(*next.borrow()) << bit_position),
                        bit_position.strict_add(1),
                    )
                });
        (bits != 0).then_some(byte)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();

        // One item is returned per 8 items in the inner iterator.
        (bytes_for_bits(lower), upper.map(bytes_for_bits))
    }

    // todo(mb): advance_by, nth
}

// If the inner iterator is [`ExactSizeIterator`], the bounds reported by
// the size hint of this iterator are exact.
impl<I, T> ExactSizeIterator for BitPacked<I, T>
where
    I: ExactSizeIterator<Item = T>,
    T: Borrow<bool>,
{
}

/// An [`Iterator`] extension trait for [`BitPacked`].
pub trait BitPackedExt<T>: Iterator<Item = T>
where
    T: Borrow<bool>,
{
    /// Packs the items in this iterator that can be borrowed as boolean values
    /// as bits in bytes using least-significant bit (LSB) numbering.
    fn bit_packed(self) -> BitPacked<Self, T>
    where
        Self: Sized,
    {
        BitPacked { iter: self.fuse() }
    }
}

impl<I, T> BitPackedExt<T> for I
where
    I: Iterator<Item = T>,
    T: Borrow<bool>,
{
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec::Vec;

    use super::*;

    #[test]
    fn iter() {
        let mut iter = [false, true, false, true, false, true].iter().bit_packed();
        assert_eq!(iter.next(), Some(0x2a));
        assert!(iter.next().is_none());
        let mut iter_byte = [false, true, false, true, false, true, false, true]
            .iter()
            .bit_packed();
        assert_eq!(iter_byte.next(), Some(0xaa));
        assert!(iter_byte.next().is_none());
        let iter_two = [true; 16].iter().bit_packed();
        assert_eq!(iter_two.collect::<Vec<u8>>(), [0xff, 0xff]);
    }

    #[test]
    fn stops_at_first_none() {
        // A non-fused iterator that yields items again after `None`.
        let input = [Some(true), None, Some(true), Some(true)];
        let mut source = input.into_iter();
        let mut iter = core::iter::from_fn(move || source.next().flatten()).bit_packed();
        assert_eq!(iter.next(), Some(0b0000_0001));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn size_hint() {
        assert_eq!((0, Some(0)), [].iter().bit_packed().size_hint());
        assert_eq!((1, Some(1)), [false].iter().bit_packed().size_hint());
        assert_eq!((1, Some(1)), [false; 7].iter().bit_packed().size_hint());
        assert_eq!((1, Some(1)), [false; 8].iter().bit_packed().size_hint());
        assert_eq!((2, Some(2)), [false; 9].iter().bit_packed().size_hint());
        assert_eq!(
            (usize::MAX / 8 + 1, None),
            (0..).map(|_| true).bit_packed().size_hint()
        );
        assert_eq!(
            (usize::MAX / 8 + 1, None),
            (0..=usize::MAX).map(|_| true).bit_packed().size_hint()
        );
        assert_eq!(
            (usize::MAX / 8 + 1, Some(usize::MAX / 8 + 1)),
            (0..usize::MAX).map(|_| true).bit_packed().size_hint()
        );
        assert_eq!((1, Some(1)), (0..3).map(|_| true).bit_packed().size_hint());
    }
}
