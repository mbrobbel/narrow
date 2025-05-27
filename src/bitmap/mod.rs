//! A collection of bits.

mod packed;
mod unpacked;

use std::{
    borrow::Borrow,
    iter::{Skip, Take},
};

use packed::BitPackedExt;
use unpacked::{BitUnpacked, BitUnpackedExt};

use crate::{
    buffer::{BufferType, VecBuffer},
    collection::{self, Collection, CollectionAlloc, CollectionRealloc, Item},
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
pub struct Bitmap<Buffer: BufferType = VecBuffer> {
    /// The bits of the bitmap are stored in this buffer of bytes.
    buffer: Buffer::Buffer<u8>,

    /// The number of bits stored in the bitmap.
    bits: usize,

    /// An offset (in number of bits) in the buffer. This enables zero-copy
    /// slicing of the bitmap on non-byte boundaries.
    offset: usize,
}

impl<Buffer: BufferType> Bitmap<Buffer> {
    /// Returns the bit index for the element at the provided index.
    /// See [`Bitmap::byte_index`].
    // TODO: clippy cast
    #[inline]
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    pub fn bit_index(&self, index: usize) -> u8 {
        ((self.offset + index) % 8) as u8
    }

    /// Returns the byte index for the element at the provided index.
    /// See [`Bitmap::bit_index`].
    #[inline]
    pub fn byte_index(&self, index: usize) -> usize {
        (self.offset + index) / 8
    }

    /// Returns the number of leading padding bits in the first byte(s) of the
    /// buffer that contain no meaningful bits. These bits should be ignored
    /// when inspecting the raw byte buffer.
    #[inline]
    pub fn leading_bits(&self) -> usize {
        self.offset
    }

    /// Returns the number of trailing padding bits in the last byte of the
    /// buffer that contain no meaningful bits. These bits should be ignored when
    /// inspecting the raw byte buffer.
    // TODO: clippy cast
    #[inline]
    #[allow(clippy::as_conversions)]
    pub fn trailing_bits(&self) -> usize {
        let trailing_bits = self.bit_index(self.bits) as usize;
        if trailing_bits == 0 {
            0
        } else {
            8 - trailing_bits
        }
    }
}

impl<Buffer: BufferType<Buffer<u8>: Clone>> Clone for Bitmap<Buffer> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            bits: self.bits,
            offset: self.offset,
        }
    }
}

impl<Buffer: BufferType<Buffer<u8>: Default>> Default for Bitmap<Buffer> {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            bits: 0,
            offset: 0,
        }
    }
}

impl<Buffer: BufferType> Length for Bitmap<Buffer> {
    fn len(&self) -> usize {
        self.bits
    }
}

impl<Buffer: BufferType> IntoIterator for Bitmap<Buffer> {
    type Item = bool;
    type IntoIter = Take<
        Skip<BitUnpacked<<<Buffer as BufferType>::Buffer<u8> as Collection<u8>>::IntoIter, u8>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer
            .into_iter()
            .bit_unpacked()
            .skip(self.offset)
            .take(self.bits)
    }
}

impl<'bitmap, Buffer: BufferType> IntoIterator for &'bitmap Bitmap<Buffer> {
    type Item = bool;
    type IntoIter = Take<
        Skip<
            BitUnpacked<<<Buffer as BufferType>::Buffer<u8> as Collection<u8>>::Iter<'bitmap>, u8>,
        >,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer
            .iter()
            .bit_unpacked()
            .skip(self.offset)
            .take(self.bits)
    }
}

impl<T: Borrow<bool>, Buffer: BufferType<Buffer<u8>: CollectionRealloc<u8>>> Extend<T>
    for Bitmap<Buffer>
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let mut additional = 0;
        self.buffer.extend(
            iter.into_iter()
                .inspect(|_| {
                    additional += 1;
                })
                .bit_packed(),
        );
        self.bits += additional;
    }
}

impl<T: Borrow<bool>, Buffer: BufferType<Buffer<u8>: CollectionAlloc<u8>>> FromIterator<T>
    for Bitmap<Buffer>
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut bits = 0;
        let buffer = iter
            .into_iter()
            .inspect(|_| {
                bits += 1;
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

impl<Buffer: BufferType> Collection<bool> for Bitmap<Buffer> {
    fn index(&self, index: usize) -> Option<<bool as collection::Item>::RefItem<'_>> {
        (index < self.len())
            .then(|| self.buffer.index(self.byte_index(index)))
            .flatten()
            .map(|byte| byte & (1 << self.bit_index(index)) != 0)
            .map(|bool| bool.as_ref_item())
    }

    type Iter<'collection>
        = <&'collection Self as IntoIterator>::IntoIter
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }

    type IntoIter = <Self as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<Buffer: BufferType<Buffer<u8>: CollectionAlloc<u8>>> CollectionAlloc<bool> for Bitmap<Buffer> {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Buffer::Buffer::<u8>::with_capacity(bytes_for_bits(capacity)),
            bits: 0,
            offset: 0,
        }
    }
}

impl<Buffer: BufferType<Buffer<u8>: CollectionRealloc<u8>>> CollectionRealloc<bool>
    for Bitmap<Buffer>
{
    fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(bytes_for_bits(additional));
    }
}

#[cfg(test)]
mod tests {
    use crate::{buffer::SliceBuffer, collection::Collection};

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
        let input = [true, false, true, true];
        let mut bitmap = Bitmap::<VecBuffer>::from_iter(input);
        bitmap.extend([false, false, false]);
        assert_eq!(bitmap.len(), 7);
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
        assert_eq!(bitmap.index(0), Some(false));
        assert_eq!(bitmap.index(1), Some(true));
        assert_eq!(bitmap.index(2), Some(false));
        assert_eq!((&bitmap).into_iter().filter(|x| !*x).count(), 2);
        assert_eq!((&bitmap).into_iter().filter(|x| *x).count(), 1);
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
