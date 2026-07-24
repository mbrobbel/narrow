//! An iterator that unpacks boolean values.

use core::{borrow::Borrow, iter::FusedIterator, ops::Range};

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
    /// A byte and the range of its bits remaining at the front.
    front: Option<(u8, Range<u8>)>,
    /// A byte and the range of its bits remaining at the back.
    back: Option<(u8, Range<u8>)>,
}

impl<I, T> Iterator for BitUnpacked<I, T>
where
    I: Iterator<Item = T>,
    T: Borrow<u8>,
{
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(front) = self.front.as_mut()
            && let Some(bit) = front.1.next()
        {
            return Some(front.0 & (1 << bit) != 0);
        }
        self.front = None;

        if let Some(item) = self.iter.next() {
            let byte = *item.borrow();
            self.front = Some((byte, 1..8));
            return Some(byte & 1 != 0);
        }

        if let Some(back) = self.back.as_mut()
            && let Some(bit) = back.1.next()
        {
            return Some(back.0 & (1 << bit) != 0);
        }
        self.back = None;
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();

        let buffered = self
            .front
            .as_ref()
            .map_or(0, |front| front.1.len())
            .strict_add(self.back.as_ref().map_or(0, |back| back.1.len()));

        // 8 items are returned per one item in the inner iterator, plus the
        // bits buffered from a partially yielded byte.
        (
            lower.saturating_mul(8).saturating_add(buffered),
            upper
                .and_then(|bound| bound.checked_mul(8))
                .and_then(|bound| bound.checked_add(buffered)),
        )
    }

    // todo(mb): advance_by, nth
}

impl<I, T> DoubleEndedIterator for BitUnpacked<I, T>
where
    I: DoubleEndedIterator<Item = T>,
    T: Borrow<u8>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(back) = self.back.as_mut()
            && let Some(bit) = back.1.next_back()
        {
            return Some(back.0 & (1 << bit) != 0);
        }
        self.back = None;

        if let Some(item) = self.iter.next_back() {
            let byte = *item.borrow();
            self.back = Some((byte, 0..7));
            return Some(byte & (1 << 7) != 0);
        }

        if let Some(front) = self.front.as_mut()
            && let Some(bit) = front.1.next_back()
        {
            return Some(front.0 & (1 << bit) != 0);
        }
        self.front = None;
        None
    }
}

// If the inner iterator is ExactSizeIterator, the bounds reported by
// the size hint of this iterator are exact.
impl<I, T> ExactSizeIterator for BitUnpacked<I, T>
where
    I: ExactSizeIterator<Item = T>,
    T: Borrow<u8>,
{
}

impl<I, T> FusedIterator for BitUnpacked<I, T>
where
    I: FusedIterator<Item = T>,
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
            front: None,
            back: None,
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
    extern crate alloc;

    use alloc::{vec, vec::Vec};

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

    #[test]
    fn size_hint_partially_yielded_byte() {
        let mut iter = [u8::MAX, 1].iter().bit_unpacked();
        let mut remaining = 16;
        assert_eq!(iter.size_hint(), (remaining, Some(remaining)));
        while iter.next().is_some() {
            remaining -= 1;
            assert_eq!(iter.size_hint(), (remaining, Some(remaining)));
        }
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    fn double_ended() {
        let mut iter = [0b1000_0001, 0b0100_0010].iter().bit_unpacked();
        assert_eq!(iter.next(), Some(true));
        assert_eq!(iter.next_back(), Some(false));
        assert_eq!(iter.next(), Some(false));
        assert_eq!(iter.next_back(), Some(true));
        assert_eq!(iter.len(), 12);
        assert_eq!(
            iter.rev().collect::<Vec<_>>(),
            [
                false, false, false, false, true, false, true, false, false, false, false, false
            ]
        );
    }
}
