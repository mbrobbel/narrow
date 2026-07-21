//! A collection of bits.

mod packed;
mod unpacked;

use core::{
    borrow::{Borrow, BorrowMut},
    fmt::{self, Debug},
    iter::{self, Skip, Take},
    slice,
};

use packed::BitPackedExt;
use unpacked::{BitUnpacked, BitUnpackedExt};

use crate::{
    buffer::{Buffer, VecBuffer},
    collection::{AllocError, Collection, CollectionAlloc, CollectionAllocIn, CollectionRealloc},
    length::Length,
};

/// Returns the number of bytes required to store the given number of bits.
#[inline]
pub(crate) const fn bytes_for_bits(bits: usize) -> usize {
    bits.div_ceil(8)
}

/// Error returned by [`Bitmap::try_from_parts`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitmapError {
    /// The requested number of `bits` at `offset` does not fit in a buffer of
    /// `bytes` bytes.
    OutOfBounds {
        /// The requested bit offset into the buffer.
        offset: usize,
        /// The requested number of bits.
        bits: usize,
        /// The number of bytes in the provided buffer.
        bytes: usize,
    },
}

impl fmt::Display for BitmapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::OutOfBounds {
                offset,
                bits,
                bytes,
            } => write!(
                f,
                "offset ({offset}) + bits ({bits}) exceeds buffer capacity ({bytes} bytes)"
            ),
        }
    }
}

impl core::error::Error for BitmapError {}

/// A collection of bits.
///
/// The validity bits are stored LSB-first in the bytes of a buffer.
/// A panicking extension leaves its committed prefix visible. The next
/// extension overwrites any uncommitted bits.
pub struct Bitmap<Storage: Buffer = VecBuffer> {
    /// The bits of the bitmap are stored in this buffer of bytes.
    ///
    /// Invariant: the buffer stores at least `bytes_for_bits(offset + bits)`
    /// bytes. Padding bits and bytes beyond the logical end are unspecified
    /// and overwritten by extensions.
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
    /// Constructs a [`Bitmap`] from its raw parts: a byte `buffer`, the number
    /// of `bits` it stores, and a bit `offset` into the buffer.
    ///
    /// # Errors
    ///
    /// Returns a [`BitmapError`] when `offset + bits` exceeds the number of
    /// bits available in the buffer (`8 * buffer.len()`).
    pub fn try_from_parts(
        buffer: Storage::For<u8>,
        bits: usize,
        offset: usize,
    ) -> Result<Self, BitmapError> {
        let bytes = buffer.borrow().len();
        let capacity = bytes.saturating_mul(8);
        match offset.checked_add(bits) {
            Some(required) if required <= capacity => Ok(Self {
                buffer,
                bits,
                offset,
            }),
            _ => Err(BitmapError::OutOfBounds {
                offset,
                bits,
                bytes,
            }),
        }
    }

    /// Returns the raw parts of this [`Bitmap`]: its byte buffer, the number of
    /// bits it stores, and the bit offset into the buffer.
    ///
    /// This is the inverse of [`Bitmap::try_from_parts`].
    #[must_use]
    pub fn into_parts(self) -> (Storage::For<u8>, usize, usize) {
        (self.buffer, self.bits, self.offset)
    }

    /// Returns the bit index for the element at the provided index.
    /// See [`Bitmap::byte_index`].
    #[inline]
    fn bit_index(&self, index: usize) -> usize {
        (self.offset.strict_add(index)).rem_euclid(8)
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
    fn trailing_bits(&self) -> usize {
        let trailing_bits = self.bit_index(self.bits);
        if trailing_bits == 0 {
            0
        } else {
            8_usize.strict_sub(trailing_bits)
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
        // Fuse to prevent a resumable iterator from writing at positions
        // beyond the first `None`.
        let mut items = iter.into_iter().fuse();

        // Fill remaining bits in the byte at the logical end of the bitmap,
        // committing the bit count per item: a panicking iterator must not
        // leave bits in the buffer that are not accounted for by the bit
        // count, because a subsequent extension could otherwise expose them.
        let bit_index = self.bit_index(self.bits);
        if bit_index != 0 {
            let byte_index = self.byte_index(self.bits);
            if let Some(byte) = self.buffer.borrow_mut().get_mut(byte_index) {
                for index in bit_index..8 {
                    if let Some(next) = items.next() {
                        // Clear the bit before setting it: bits beyond the
                        // logical end of the bitmap are unspecified.
                        *byte = (*byte & !(1 << index)) | (u8::from(*next.borrow()) << index);
                        self.bits = self.bits.strict_add(1);
                    }
                }
            }
        }

        // Use bit packed iterator for the remainder. The bit count is
        // committed only when the extension completes; a panic leaves the
        // written bytes beyond the logical end to be overwritten later.
        if let Some(first) = items.next() {
            let mut consumed: usize = 0;
            let mut packed = iter::once(first)
                .chain(items)
                .inspect(|_| consumed = consumed.strict_add(1))
                .bit_packed();

            // Overwrite bytes beyond the logical end before appending.
            let mut byte_index = self.byte_index(self.bits);
            while let Some(byte) = self.buffer.borrow_mut().get_mut(byte_index) {
                match packed.next() {
                    Some(packed_byte) => {
                        *byte = packed_byte;
                        byte_index = byte_index.strict_add(1);
                    }
                    None => break,
                }
            }

            self.buffer.extend(packed);
            self.bits = self.bits.strict_add(consumed);
        }
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

impl<Storage: Buffer<For<u8>: CollectionAllocIn>> CollectionAllocIn for Bitmap<Storage> {
    type Alloc = <Storage::For<u8> as CollectionAllocIn>::Alloc;

    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self {
        Self {
            buffer: Storage::For::<u8>::with_capacity_in(bytes_for_bits(capacity), alloc),
            bits: 0,
            offset: 0,
        }
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, alloc: Self::Alloc) -> Self {
        let mut bits: usize = 0;
        let buffer = Storage::For::<u8>::from_iter_in(
            iter.into_iter()
                .inspect(|_| bits = bits.strict_add(1))
                .bit_packed(),
            alloc,
        );
        Self {
            buffer,
            bits,
            offset: 0,
        }
    }

    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, AllocError> {
        Storage::For::<u8>::try_with_capacity_in(bytes_for_bits(capacity), alloc).map(|buffer| {
            Self {
                buffer,
                bits: 0,
                offset: 0,
            }
        })
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, AllocError> {
        let mut bits: usize = 0;
        let buffer = Storage::For::<u8>::try_from_iter_in(
            iter.into_iter()
                .inspect(|_| bits = bits.strict_add(1))
                .bit_packed(),
            alloc,
        )?;
        Ok(Self {
            buffer,
            bits,
            offset: 0,
        })
    }
}

impl<Storage: Buffer<For<u8>: BorrowMut<[u8]> + CollectionRealloc>> CollectionRealloc
    for Bitmap<Storage>
{
    fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError> {
        if let Some(bits) = additional.checked_sub(self.trailing_bits()) {
            self.buffer.try_reserve(bytes_for_bits(bits))?;
        }
        Ok(())
    }

    fn try_extend<I: IntoIterator<Item = Self::Owned>>(
        &mut self,
        iter: I,
    ) -> Result<(), AllocError> {
        let original_bits = self.bits;
        let mut items = iter.into_iter().fuse();

        let bit_index = self.bit_index(self.bits);
        if bit_index != 0 {
            let byte_index = self.byte_index(self.bits);
            if let Some(byte) = self.buffer.borrow_mut().get_mut(byte_index) {
                for index in bit_index..8 {
                    if let Some(next) = items.next() {
                        *byte = (*byte & !(1 << index)) | (u8::from(next) << index);
                        self.bits = self.bits.strict_add(1);
                    }
                }
            }
        }

        if let Some(first) = items.next() {
            let mut consumed: usize = 0;
            let mut packed = iter::once(first)
                .chain(items)
                .inspect(|_| consumed = consumed.strict_add(1))
                .bit_packed();

            let mut byte_index = self.byte_index(self.bits);
            while let Some(byte) = self.buffer.borrow_mut().get_mut(byte_index) {
                match packed.next() {
                    Some(packed_byte) => {
                        *byte = packed_byte;
                        byte_index = byte_index.strict_add(1);
                    }
                    None => break,
                }
            }

            if let Err(error) = self.buffer.try_extend(packed) {
                self.bits = original_bits;
                return Err(error);
            }
            self.bits = self.bits.strict_add(consumed);
        }
        Ok(())
    }

    fn reserve(&mut self, additional: usize) {
        if let Some(bits) = additional.checked_sub(self.trailing_bits()) {
            self.buffer.reserve(bytes_for_bits(bits));
        }
    }

    fn truncate(&mut self, len: usize) {
        if len < self.bits {
            // Bits beyond the logical end are unspecified; a subsequent
            // extension overwrites them (see the buffer invariant).
            self.bits = len;
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
    fn bytes_for_bits_does_not_saturate_before_rounding() {
        assert_eq!(bytes_for_bits(usize::MAX), usize::MAX / 8 + 1);
    }

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

    #[test]
    fn extend_panic_safety() {
        extern crate std;
        use std::panic::{AssertUnwindSafe, catch_unwind};

        let mut bitmap = Bitmap::<VecBuffer>::from_iter([true]);
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                bitmap
                    .extend(core::iter::once(true).chain(core::iter::once_with(|| panic!("boom"))));
            }))
            .is_err()
        );

        // The item yielded before the panic is committed.
        assert_eq!(bitmap.len(), 2);
        assert_eq!(bitmap.view(1), Some(true));

        // A subsequent extension observes a consistent state.
        bitmap.extend([false, true]);
        assert_eq!(bitmap.len(), 4);
        assert_eq!(
            bitmap.iter_views().collect::<Vec<_>>(),
            [true, true, false, true]
        );
    }

    #[test]
    fn extend_panic_discards_uncommitted_packed_bits() {
        extern crate std;
        use std::panic::{AssertUnwindSafe, catch_unwind};

        let mut bitmap = Bitmap::<VecBuffer>::default();
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                // Panics after yielding 10 items: the interrupted extension
                // is not committed, the bytes it wrote are unspecified data
                // beyond the logical end of the bitmap.
                bitmap.extend(
                    (0..10)
                        .map(|i| i % 2 == 0)
                        .chain(core::iter::once_with(|| panic!("boom"))),
                );
            }))
            .is_err()
        );

        assert_eq!(bitmap.len(), 0);
        assert_eq!(bitmap.view(0), None);

        bitmap.extend([false, true, false]);
        assert_eq!(bitmap.len(), 3);
        assert_eq!(
            bitmap.iter_views().collect::<Vec<_>>(),
            [false, true, false]
        );
    }

    #[test]
    fn extend_non_fused_iterator() {
        /// Yields `Some(true)` again after the first `None`.
        struct Resumable(usize);
        impl Iterator for Resumable {
            type Item = bool;
            fn next(&mut self) -> Option<bool> {
                self.0 = self.0.strict_add(1);
                match self.0 {
                    1 => Some(false),
                    2 => None,
                    _ => Some(true),
                }
            }
        }

        let mut bitmap = Bitmap::<VecBuffer>::from_iter([true]);
        bitmap.extend(Resumable(0));
        assert_eq!(bitmap.len(), 2);
        assert_eq!(bitmap.iter_views().collect::<Vec<_>>(), [true, false]);
    }

    #[test]
    fn extend_clears_stale_bits() {
        // Stale bits beyond the bit count, as left behind by an interrupted
        // extension.
        let mut bitmap: Bitmap<VecBuffer> = Bitmap {
            buffer: alloc::vec![0b1111_1111],
            bits: 4,
            offset: 0,
        };
        bitmap.extend([false, true]);
        assert_eq!(bitmap.len(), 6);
        assert_eq!(bitmap.view(4), Some(false));
        assert_eq!(bitmap.view(5), Some(true));
        assert_eq!(bitmap.buffer.as_slice(), &[0b1110_1111]);
    }

    #[test]
    fn into_iter_len() {
        let bitmap = Bitmap::<VecBuffer>::from_iter([true, false, true, true]);
        let mut iter = bitmap.into_iter_owned();
        let mut remaining = 4;
        assert_eq!(iter.len(), remaining);
        while iter.next().is_some() {
            remaining -= 1;
            assert_eq!(iter.len(), remaining);
        }
        assert_eq!(remaining, 0);
    }

    #[test]
    fn try_from_parts() {
        let bitmap = Bitmap::<VecBuffer>::try_from_parts(alloc::vec![0b0000_1101], 4, 0)
            .expect("valid parts");
        assert_eq!(bitmap.len(), 4);
        assert_eq!(bitmap.view(0), Some(true));
        assert_eq!(bitmap.view(1), Some(false));
        assert_eq!(bitmap.view(2), Some(true));
        assert_eq!(bitmap.view(3), Some(true));
        let (buffer, bits, offset) = bitmap.into_parts();
        assert_eq!(buffer, alloc::vec![0b0000_1101]);
        assert_eq!(bits, 4);
        assert_eq!(offset, 0);
    }

    #[test]
    fn try_from_parts_out_of_bounds() {
        let error = Bitmap::<VecBuffer>::try_from_parts(alloc::vec![0_u8], 5, 4)
            .expect_err("out of bounds");
        assert_eq!(
            error,
            BitmapError::OutOfBounds {
                offset: 4,
                bits: 5,
                bytes: 1,
            }
        );
    }

    #[test]
    fn truncate() {
        let mut bitmap = Bitmap::<VecBuffer>::from_iter([true, false, true, true]);
        bitmap.truncate(2);
        assert_eq!(bitmap.len(), 2);
        assert_eq!(bitmap.iter_views().collect::<Vec<_>>(), [true, false]);
        // Truncating beyond the length is a no-op.
        bitmap.truncate(5);
        assert_eq!(bitmap.len(), 2);
        // A subsequent extension overwrites the stale bits.
        bitmap.extend([false, false]);
        assert_eq!(
            bitmap.iter_views().collect::<Vec<_>>(),
            [true, false, false, false]
        );
    }

    #[test]
    fn into_iter_len_offset() {
        let bitmap: Bitmap<SliceBuffer> = Bitmap {
            buffer: &[0b1010_0000, 0b0000_0101],
            bits: 7,
            offset: 4,
        };
        let mut iter = bitmap.into_iter_owned();
        let mut remaining = 7;
        assert_eq!(iter.len(), remaining);
        while iter.next().is_some() {
            remaining -= 1;
            assert_eq!(iter.len(), remaining);
        }
        assert_eq!(remaining, 0);
    }
}
