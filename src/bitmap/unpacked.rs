//! An iterator that unpacks boolean values.

use std::borrow::Borrow;

/// An iterator that unpacks boolean values from an iterator (`I`) over items
/// (`T`) that can be borrowed as bytes, by interpreting the bits of these bytes
/// with least-significant bit (LSB) numbering as boolean values i.e. `1` maps
/// to `true` and `0` maps to `false`.
///
// note: add to docs that users should combine this with std::iter::skip and
// std::iter::take if needed for padding
#[derive(Debug)]
pub struct BitUnpacked<I, T>
where
    I: Iterator<Item = T>,
    T: Borrow<u8>,
{
    /// The iterator over the bytes storing packed bits.
    iter: I,
    /// The popped byte yielding bits.
    byte: Option<u8>,
    /// The mask selecting bits from the popped byte.
    mask: u8,
}

impl<I, T> Iterator for BitUnpacked<I, T>
where
    I: Iterator<Item = T>,
    T: Borrow<u8>,
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
            upper.and_then(|bound| bound.checked_mul(8)),
        )
    }

    // todo(mb): advance_by, nth
}

// If the inner iterator is ExactSizeIterator, the bounds reported by
// the size hint of this iterator are exact.
impl<I, T> ExactSizeIterator for BitUnpacked<I, T>
where
    I: ExactSizeIterator<Item = T>,
    T: Borrow<u8>,
{
}

/// An [`Iterator`] extension trait for [`BitUnpacked`].
pub trait BitUnpackedExt<T>: IntoIterator<Item = T>
where
    T: Borrow<u8>,
{
    /// Returns an iterator that unpacks bits from the bytes in the iterator.
    fn bit_unpacked(self) -> BitUnpacked<Self::IntoIter, T>
    where
        Self: Sized,
    {
        BitUnpacked {
            iter: self.into_iter(),
            byte: None,
            mask: 0x01,
        }
    }
}

impl<I, T> BitUnpackedExt<T> for I
where
    I: IntoIterator<Item = T>,
    T: Borrow<u8>,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter() {
        let iter = [u8::MAX, 1].iter().bit_unpacked();
        assert_eq!(
            iter.collect::<Vec<_>>(),
            vec![
                true, true, true, true, true, true, true, true, true, false, false, false, false,
                false, false, false
            ]
        );
    }

    #[test]
    fn size_hint() {
        let input = [u8::MAX, 1, 2, 3];
        assert_eq!(
            input.iter().bit_unpacked().size_hint(),
            (input.len() * 8, Some(input.len() * 8))
        );
    }
}
