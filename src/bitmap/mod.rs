//! A collection of bits.

mod packed;
mod unpacked;

use core::{
    borrow::{Borrow, BorrowMut},
    fmt::{self, Debug},
    iter::{Skip, Take},
    slice,
};

use packed::BitPackedExt;
use unpacked::{BitUnpacked, BitUnpackedExt};

use crate::{
    buffer::{Buffer, VecBuffer},
    collection::{Collection, CollectionAlloc, CollectionRealloc},
    length::Length,
};

/// Returns the number of bytes required to store the given number of bits.
#[inline]
pub(crate) const fn bytes_for_bits(bits: usize) -> usize {
    bits.saturating_add(7) / 8
}

/// A collection of bits.
///
/// The validity bits are stored LSB-first in the bytes of a buffer.
pub struct Bitmap<Storage: Buffer = VecBuffer> {
    /// The bits of the bitmap are stored in this buffer of bytes.
    buffer: Storage::For<u8>,

    /// The number of bits stored in the bitmap.
    bits: usize,

    /// An offset (in number of bits) in the buffer. This enables zero-copy
    /// slicing of the bitmap on non-byte boundaries.
    offset: usize,
}

impl<Storage: Buffer<For<u8>: Debug>> Debug for Bitmap<Storage> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bitmap")
            .field("buffer", &self.buffer)
            .field("bits", &self.bits)
            .field("offset", &self.offset)
            .finish()
    }
}

impl<Storage: Buffer> Bitmap<Storage> {
    /// Returns the bit index for the element at the provided index.
    /// See [`Bitmap::byte_index`].
    #[inline]
    fn bit_index(&self, index: usize) -> u8 {
        // As conversion because 0 <= remainder < 8 < u8::MAX
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        {
            self.offset.rem_euclid(8).strict_add(index.rem_euclid(8)) as u8
        }
    }

    /// Returns the byte index for the element at the provided index.
    /// See [`Bitmap::bit_index`].
    ///
    /// # Panics
    ///
    /// This function panics on overflow of the index and offset addition.
    #[inline]
    fn byte_index(&self, index: usize) -> usize {
        self.leading_bits().strict_add(index).strict_div(8)
    }

    /// Returns the number of leading padding bits in the first byte(s) of the
    /// buffer that contain no meaningful bits. These bits should be ignored
    /// when inspecting the raw byte buffer.
    #[inline]
    fn leading_bits(&self) -> usize {
        self.offset
    }

    /// Returns the number of trailing padding bits in the last byte of the
    /// buffer that contain no meaningful bits. These bits should be ignored when
    /// inspecting the raw byte buffer.
    #[inline]
    fn trailing_bits(&self) -> u8 {
        let trailing_bits = self.bit_index(self.bits);
        if trailing_bits == 0 {
            0
        } else {
            8_u8.strict_sub(trailing_bits)
        }
    }
}

impl<Storage: Buffer<For<u8>: Clone>> Clone for Bitmap<Storage> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            bits: self.bits,
            offset: self.offset,
        }
    }
}

impl<Storage: Buffer<For<u8>: Default>> Default for Bitmap<Storage> {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            bits: 0,
            offset: 0,
        }
    }
}

impl<Storage: Buffer> Length for Bitmap<Storage> {
    fn len(&self) -> usize {
        self.bits
    }
}

impl<Storage: Buffer> IntoIterator for Bitmap<Storage> {
    type Item = bool;

    type IntoIter = Take<Skip<BitUnpacked<<Storage::For<u8> as Collection>::IntoIter, u8>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer
            .into_iter_owned()
            .bit_unpacked()
            .skip(self.offset)
            .take(self.bits)
    }
}

impl<'bitmap, Storage: Buffer> IntoIterator for &'bitmap Bitmap<Storage> {
    type Item = bool;
    type IntoIter = Take<Skip<BitUnpacked<slice::Iter<'bitmap, u8>, &'bitmap u8>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer
            .borrow()
            .iter()
            .bit_unpacked()
            .skip(self.offset)
            .take(self.bits)
    }
}

impl<T: Borrow<bool>, Storage: Buffer<For<u8>: BorrowMut<[u8]> + CollectionRealloc>> Extend<T>
    for Bitmap<Storage>
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        // Track the number of added bits.
        let mut additional: usize = 0;
        let mut items = iter.into_iter().inspect(|_| {
            additional = additional.strict_add(1);
        });

        // Fill remaining bits in the last byte of the buffer.
        let bit_index = self.bit_index(self.bits);
        if bit_index != 0 {
            // If there are trailing bits, there must at least be one byte in
            // the buffer.
            if let Some(last_byte) = self.buffer.borrow_mut().last_mut() {
                // Use the remaining bits in this last byte
                for index in bit_index..8 {
                    if let Some(next) = items.next() {
                        *last_byte |= u8::from(*next.borrow()) << index;
                    }
                }
            }
        }

        // Use bit packed iterator for the remainder
        self.buffer.extend(items.bit_packed());
        self.bits = self.bits.strict_add(additional);
    }
}

impl<T: Borrow<bool>, Storage: Buffer<For<u8>: CollectionAlloc>> FromIterator<T>
    for Bitmap<Storage>
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut bits: usize = 0;
        let buffer = iter
            .into_iter()
            .inspect(|_| {
                bits = bits.strict_add(1);
            })
            .bit_packed()
            .collect();
        Self {
            buffer,
            bits,
            offset: 0,
        }
    }
}

impl<Storage: Buffer> Collection for Bitmap<Storage> {
    type View<'collection>
        = bool
    where
        Self: 'collection;

    type Owned = bool;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        (index < self.len())
            .then(|| self.buffer.borrow().view(self.byte_index(index)))
            .flatten()
            .map(|byte| byte & (1 << self.bit_index(index)) != 0)
    }

    type Iter<'collection>
        = <&'collection Self as IntoIterator>::IntoIter
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }

    type IntoIter = <Self as IntoIterator>::IntoIter;

    fn into_iter_owned(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<Storage: Buffer<For<u8>: CollectionAlloc>> CollectionAlloc for Bitmap<Storage> {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Storage::For::<u8>::with_capacity(bytes_for_bits(capacity)),
            bits: 0,
            offset: 0,
        }
    }
}

impl<Storage: Buffer<For<u8>: BorrowMut<[u8]> + CollectionRealloc>> CollectionRealloc
    for Bitmap<Storage>
{
    fn reserve(&mut self, additional: usize) {
        if let Some(bits) = additional.checked_sub(usize::from(self.trailing_bits())) {
            self.buffer.reserve(bytes_for_bits(bits));
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use crate::buffer::SliceBuffer;
    use alloc::vec::Vec;

    use super::*;

    #[test]
    fn from_iter() {
        let input = [true, false, true, true];
        let bitmap = Bitmap::<VecBuffer>::from_iter(input);
        assert_eq!(
            input,
            IntoIterator::into_iter(bitmap)
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn extend() {
        let mut bitmap = Bitmap::<VecBuffer>::from_iter([true]);
        assert_eq!(bitmap.buffer, &[0b0000_0001]);
        bitmap.extend([true]);
        assert_eq!(bitmap.buffer, &[0b0000_0011]);
    }

    #[test]
    fn extend_within_byte() {
        let input = [true, false, true, true];
        let mut bitmap = Bitmap::<VecBuffer>::from_iter(input);
        assert_eq!(bitmap.len(), 4);
        assert_eq!(bitmap.buffer.len(), 1);
        assert_eq!(bitmap.buffer.as_slice(), &[0b0000_1101]);

        bitmap.extend([true, false, false]);
        assert_eq!(bitmap.len(), 7);
        assert_eq!(bitmap.buffer.len(), 1);
        assert_eq!(bitmap.buffer.as_slice(), &[0b0001_1101]);

        bitmap.extend([true]);
        assert_eq!(bitmap.len(), 8);
        assert_eq!(bitmap.buffer.len(), 1);
        assert_eq!(bitmap.buffer.as_slice(), &[0b1001_1101]);
    }

    #[test]
    fn extend_within_byte_with_offset() {
        let input = [true, false, true, true];
        let mut bitmap = Bitmap::<VecBuffer>::from_iter(input);
        bitmap.offset = 2;
        bitmap.bits = 2;
        assert_eq!(bitmap.len(), 2);
        assert_eq!(bitmap.buffer.len(), 1);
        assert_eq!(bitmap.buffer.as_slice(), &[0b0000_1101]);

        bitmap.extend([true, false, false, true, true, true]);
        assert_eq!(bitmap.len(), 8);
        assert_eq!(bitmap.buffer.len(), 2);
        assert_eq!(bitmap.buffer.as_slice(), &[0b1001_1101, 0b0000_0011]);
    }

    #[test]
    fn extend_across_next_byte() {
        let input = [true; 8];
        let mut bitmap = Bitmap::<VecBuffer>::from_iter(input);
        assert_eq!(bitmap.len(), 8);
        assert_eq!(bitmap.buffer.len(), 1);
        assert_eq!(bitmap.buffer.as_slice(), &[0b1111_1111]);

        bitmap.extend([true]);
        assert_eq!(bitmap.len(), 9);
        assert_eq!(bitmap.buffer.len(), 2);
        assert_eq!(bitmap.buffer.as_slice(), &[0b1111_1111, 0b0000_0001]);
    }

    #[test]
    fn slice_buffer() {
        let slice = &[0b1010_0000];
        let bitmap: Bitmap<SliceBuffer> = Bitmap {
            buffer: slice,
            bits: 3,
            offset: 4,
        };
        assert_eq!(bitmap.len(), 3);
        assert_eq!(bitmap.leading_bits(), 4);
        assert_eq!(bitmap.trailing_bits(), 1);
        assert_eq!(bitmap.view(0), Some(false));
        assert_eq!(bitmap.view(1), Some(true));
        assert_eq!(bitmap.view(2), Some(false));
        assert_eq!(bitmap.view(3), None);
        assert_eq!(bitmap.iter_views().filter(|x| !*x).count(), 2);
        assert_eq!(bitmap.iter_views().filter(|x| *x).count(), 1);
        assert_eq!(
            IntoIterator::into_iter(bitmap).collect::<Vec<_>>(),
            [false, true, false]
        );
    }

    #[test]
    fn alloc() {
        let bitmap = Bitmap::<VecBuffer>::with_capacity(15);
        assert_eq!(bitmap.buffer.capacity(), 2);
    }
}
