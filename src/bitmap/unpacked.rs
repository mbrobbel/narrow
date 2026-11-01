//! An iterator that unpacks boolean values.

use core::{borrow::Borrow, iter::FusedIterator, num::NonZeroU16, ops::Range};

/// An iterator over a borrowed bitmap's logical bits in LSB-first order.
#[derive(Clone, Debug, Default)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct BitmapIter<'a> {
    bytes: &'a [u8],
    indices: Range<usize>,
}

impl<'a> BitmapIter<'a> {
    /// Creates an iterator over the bits at `indices` in `bytes`.
    ///
    /// Callers must ensure every index in `indices` addresses a bit within
    /// `bytes`; [`Bitmap`](super::Bitmap) guarantees this for its offset and
    /// length. Out-of-range indices panic when yielded.
    pub(super) fn new(bytes: &'a [u8], indices: Range<usize>) -> Self {
        debug_assert!(
            indices.is_empty() || indices.end.div_ceil(8) <= bytes.len(),
            "bit range {indices:?} exceeds {} bytes",
            bytes.len()
        );
        Self { bytes, indices }
    }

    #[inline]
    fn bit(&self, index: usize) -> bool {
        bit(self.bytes, index)
    }
}

/// Number of bits packed in one byte.
const BITS_PER_BYTE: usize = 8;

/// Marker bit placed above a freshly fetched byte in [`BitUnpacked`].
const SENTINEL: u16 = 1 << BITS_PER_BYTE;

/// Returns the bit at `index` in `bytes`, using LSB-first numbering.
#[inline]
fn bit(bytes: &[u8], index: usize) -> bool {
    bytes[index.strict_div(BITS_PER_BYTE)] & (1 << index.rem_euclid(BITS_PER_BYTE)) != 0
}

impl Iterator for BitmapIter<'_> {
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().map(|index| self.bit(index))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.indices.nth(n).map(|index| self.bit(index))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }

    #[inline]
    fn fold<B, F: FnMut(B, Self::Item) -> B>(self, init: B, mut f: F) -> B {
        let Self { bytes, indices } = self;
        indices.fold(init, |acc, index| f(acc, bit(bytes, index)))
    }

    /// Collects through `Map<Range<usize>, _>`, which implements the unstable
    /// `TrustedLen` trait, so collections such as `Vec` can allocate once and
    /// write without per-item capacity checks. `BitmapIter` itself cannot
    /// implement `TrustedLen` on stable Rust.
    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        let Self { bytes, indices } = self;
        indices.map(|index| bit(bytes, index)).collect()
    }
}

impl DoubleEndedIterator for BitmapIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.indices.next_back().map(|index| self.bit(index))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.indices.nth_back(n).map(|index| self.bit(index))
    }
}

impl ExactSizeIterator for BitmapIter<'_> {}
impl FusedIterator for BitmapIter<'_> {}

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
    /// The unread bits of the current byte, below a sentinel set bit.
    ///
    /// The next bit to yield is the least-significant bit. `bits.ilog2()` is
    /// the number of unread bits, so `1` (only the sentinel) means the current
    /// byte is exhausted and the next byte must be fetched.
    bits: NonZeroU16,
}

impl<I, T> Iterator for BitUnpacked<I, T>
where
    I: Iterator<Item = T>,
    T: Borrow<u8>,
{
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let pending = if self.bits == NonZeroU16::MIN {
            u16::from(*self.iter.next()?.borrow()) | SENTINEL
        } else {
            self.bits.get()
        };
        self.bits = NonZeroU16::new(pending >> 1).expect("sentinel remains set");
        Some(pending & 1 != 0)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();

        let buffered = usize::try_from(self.bits.ilog2()).expect("bit count fits in usize");

        // 8 items are returned per one item in the inner iterator, plus the
        // bits buffered from a partially yielded byte.
        (
            lower.saturating_mul(8).saturating_add(buffered),
            upper
                .and_then(|bound| bound.checked_mul(8))
                .and_then(|bound| bound.checked_add(buffered)),
        )
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let buffered = usize::try_from(self.bits.ilog2()).expect("bit count fits in usize");
        if n < buffered {
            self.bits = NonZeroU16::new(self.bits.get() >> n).expect("sentinel remains set");
        } else {
            let remaining = n.strict_sub(buffered);
            self.bits = NonZeroU16::MIN;
            let byte = self.iter.nth(remaining.strict_div(BITS_PER_BYTE))?;
            let pending =
                (u16::from(*byte.borrow()) | SENTINEL) >> remaining.rem_euclid(BITS_PER_BYTE);
            self.bits = NonZeroU16::new(pending).expect("sentinel remains set");
        }
        self.next()
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
            bits: NonZeroU16::MIN,
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
    fn nth_matches_repeated_next() {
        let input = [0x00, 0xff, 0xa5, 0x3c];
        for consumed in 0..=32 {
            for skip in 0..=40 {
                let mut expected = input.iter().bit_unpacked();
                let mut actual = input.iter().bit_unpacked();
                for _ in 0..consumed {
                    expected.next();
                    actual.next();
                }
                for _ in 0..skip {
                    expected.next();
                }
                assert_eq!(actual.nth(skip), expected.next());
                assert_eq!(actual.size_hint(), expected.size_hint());
                assert_eq!(actual.collect::<Vec<_>>(), expected.collect::<Vec<_>>());
            }
        }
    }

    #[test]
    fn resumes_after_none() {
        let input = [Some(0xa5), None, Some(0x3c)];
        let mut source = input.into_iter();
        let mut iter = core::iter::from_fn(move || source.next().flatten()).bit_unpacked();
        assert_eq!(iter.nth(7), Some(true));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), Some(false));
        assert_eq!(iter.nth(1), Some(true));
        assert_eq!(iter.nth(usize::MAX), None);
    }

    #[test]
    fn default_bitmap_iter_is_empty() {
        assert_eq!(BitmapIter::default().next(), None);
        assert_eq!(BitmapIter::default().len(), 0);
    }

    #[test]
    fn borrowed_iteration_and_skipping() {
        let bytes = [0x00, 0xff, 0xa5, 0x3c, 0x81, 0x7e];
        for start in 0..=48 {
            for end in start..=48 {
                let expected: Vec<_> = (start..end)
                    .map(|bit| bytes[bit / 8] & (1 << (bit % 8)) != 0)
                    .collect();
                let mut iter = BitmapIter::new(&bytes, start..end);
                assert_eq!(iter.clone().collect::<Vec<_>>(), expected);
                assert_eq!(
                    iter.clone().fold(Vec::new(), |mut output, bit| {
                        output.push(bit);
                        output
                    }),
                    expected
                );
                for (index, &bit) in expected.iter().enumerate() {
                    assert_eq!(iter.len(), expected.len() - index);
                    assert_eq!(iter.next(), Some(bit));
                }
                assert_eq!(iter.size_hint(), (0, Some(0)));
                assert_eq!(iter.next(), None);
                assert_eq!(iter.next(), None);

                assert_eq!(
                    BitmapIter::new(&bytes, start..end)
                        .rev()
                        .collect::<Vec<_>>(),
                    expected.iter().rev().copied().collect::<Vec<_>>()
                );
                let mut back = BitmapIter::new(&bytes, start..end);
                assert_eq!(
                    back.nth_back(7),
                    expected.len().checked_sub(8).map(|index| expected[index])
                );
                assert_eq!(back.len(), expected.len().saturating_sub(8));
                assert_eq!(
                    back.next(),
                    expected.first().copied().filter(|_| expected.len() > 8)
                );

                let mut skipped = BitmapIter::new(&bytes, start..end);
                assert_eq!(skipped.nth(7), expected.get(7).copied());
                assert_eq!(skipped.len(), expected.len().saturating_sub(8));
                assert_eq!(skipped.nth(usize::MAX), None);
                assert_eq!(skipped.len(), 0);
            }
        }
    }
}
