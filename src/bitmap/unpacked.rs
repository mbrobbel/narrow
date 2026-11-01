//! An iterator that unpacks boolean values.

use core::{borrow::Borrow, iter::FusedIterator, ops::Range};

use super::Bitmap;
use crate::buffer::Buffer;

/// An iterator over a bitmap's logical bits in LSB-first order.
///
/// `Bytes` is the byte buffer: a borrowed slice when iterating over a
/// `&Bitmap`, or the bitmap's own buffer when iterating over a `Bitmap` by
/// value.
///
/// It can only be created from a [`Bitmap`] (or empty, with [`Default`]), so
/// its indices always address bits within its bytes.
#[derive(Clone, Debug, Default)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct BitmapIter<Bytes> {
    /// The packed bits.
    bytes: Bytes,
    /// The indices of the bits that remain to be yielded.
    indices: Range<usize>,
}

impl<Storage: Buffer> IntoIterator for Bitmap<Storage> {
    type Item = bool;
    type IntoIter = BitmapIter<Storage::For<u8>>;

    fn into_iter(self) -> Self::IntoIter {
        BitmapIter {
            indices: self.offset..self.offset.strict_add(self.bits),
            bytes: self.buffer,
        }
    }
}

impl<'bitmap, Storage: Buffer> IntoIterator for &'bitmap Bitmap<Storage> {
    type Item = bool;
    type IntoIter = BitmapIter<&'bitmap [u8]>;

    fn into_iter(self) -> Self::IntoIter {
        BitmapIter {
            bytes: self.buffer.borrow(),
            indices: self.offset..self.offset.strict_add(self.bits),
        }
    }
}

impl<Bytes: Borrow<[u8]>> BitmapIter<Bytes> {
    #[inline]
    fn bit(&self, index: usize) -> bool {
        bit(self.bytes.borrow(), index)
    }
}

/// Number of bits packed in one byte.
const BITS_PER_BYTE: usize = 8;

/// Returns the bit at `index` in `bytes`, using LSB-first numbering.
#[inline]
fn bit(bytes: &[u8], index: usize) -> bool {
    bytes[index.strict_div(BITS_PER_BYTE)] & (1 << index.rem_euclid(BITS_PER_BYTE)) != 0
}

impl<Bytes: Borrow<[u8]>> Iterator for BitmapIter<Bytes> {
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
    fn count(self) -> usize {
        self.indices.len()
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    /// Folds byte by byte: the bits of each whole byte are extracted with
    /// constant shifts instead of a division, bounds check and variable shift
    /// per bit, which lets adapters such as `filter(..).count()` vectorize.
    #[inline]
    fn fold<B, F: FnMut(B, Self::Item) -> B>(self, init: B, mut f: F) -> B {
        let Self {
            bytes: buffer,
            indices: Range { start, end },
        } = self;
        let bytes = buffer.borrow();

        // Leading bits up to the first byte boundary.
        let head_end = start.next_multiple_of(BITS_PER_BYTE).min(end);
        let head = (start..head_end).fold(init, |acc, index| f(acc, bit(bytes, index)));

        // Whole bytes. `head_end` is byte aligned whenever bits remain.
        let body_end = end.strict_div(BITS_PER_BYTE);
        let body_start = head_end.strict_div(BITS_PER_BYTE).min(body_end);
        let body = bytes[body_start..body_end].iter().fold(head, |acc, &byte| {
            (0..u8::BITS).fold(acc, |partial, shift| f(partial, byte & (1 << shift) != 0))
        });

        // Trailing bits after the last byte boundary.
        let tail_start = body_end.strict_mul(BITS_PER_BYTE).max(head_end);
        (tail_start..end).fold(body, |acc, index| f(acc, bit(bytes, index)))
    }

    /// Collects through `Map<Range<usize>, _>`, which implements the unstable
    /// `TrustedLen` trait, so collections such as `Vec` can allocate once and
    /// write without per-item capacity checks. `BitmapIter` itself cannot
    /// implement `TrustedLen` on stable Rust.
    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        let Self { bytes, indices } = self;
        indices.map(|index| bit(bytes.borrow(), index)).collect()
    }
}

impl<Bytes: Borrow<[u8]>> DoubleEndedIterator for BitmapIter<Bytes> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.indices.next_back().map(|index| self.bit(index))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.indices.nth_back(n).map(|index| self.bit(index))
    }
}

impl<Bytes: Borrow<[u8]>> ExactSizeIterator for BitmapIter<Bytes> {}
impl<Bytes: Borrow<[u8]>> FusedIterator for BitmapIter<Bytes> {}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec::Vec;

    use super::*;
    use crate::buffer::VecBuffer;

    /// Returns a bitmap over the bits at `indices` in `bytes`.
    fn bitmap(bytes: &[u8], indices: Range<usize>) -> Bitmap<VecBuffer> {
        Bitmap {
            buffer: bytes.to_vec(),
            bits: indices.len(),
            offset: indices.start,
        }
    }

    #[test]
    fn default_bitmap_iter_is_empty() {
        assert_eq!(BitmapIter::<&[u8]>::default().next(), None);
        assert_eq!(BitmapIter::<Vec<u8>>::default().len(), 0);
    }

    #[test]
    fn borrowed_iteration_and_skipping() {
        let bytes = [0x00, 0xff, 0xa5, 0x3c, 0x81, 0x7e];
        for start in 0..=48 {
            for end in start..=48 {
                let expected: Vec<_> = (start..end)
                    .map(|bit| bytes[bit / 8] & (1 << (bit % 8)) != 0)
                    .collect();
                let bitmap = bitmap(&bytes, start..end);
                let owned = bitmap.clone().into_iter();
                assert_eq!(owned.clone().collect::<Vec<_>>(), expected);
                assert_eq!(
                    owned.filter(|bit| *bit).count(),
                    expected.iter().filter(|bit| **bit).count()
                );

                let mut iter = (&bitmap).into_iter();
                assert_eq!(iter.clone().collect::<Vec<_>>(), expected);
                assert_eq!(iter.clone().count(), expected.len());
                assert_eq!(iter.clone().last(), expected.last().copied());
                assert_eq!(
                    iter.clone().filter(|bit| *bit).count(),
                    expected.iter().filter(|bit| **bit).count()
                );
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
                    (&bitmap).into_iter().rev().collect::<Vec<_>>(),
                    expected.iter().rev().copied().collect::<Vec<_>>()
                );
                let mut back = (&bitmap).into_iter();
                assert_eq!(
                    back.nth_back(7),
                    expected.len().checked_sub(8).map(|index| expected[index])
                );
                assert_eq!(back.len(), expected.len().saturating_sub(8));
                assert_eq!(
                    back.next(),
                    expected.first().copied().filter(|_| expected.len() > 8)
                );

                let mut skipped = (&bitmap).into_iter();
                assert_eq!(skipped.nth(7), expected.get(7).copied());
                assert_eq!(skipped.len(), expected.len().saturating_sub(8));
                assert_eq!(skipped.nth(usize::MAX), None);
                assert_eq!(skipped.len(), 0);
            }
        }
    }
}
