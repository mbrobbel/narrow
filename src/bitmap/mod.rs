//! A collection of bits.

use std::{
    any,
    borrow::Borrow,
    fmt::{Debug, Formatter, Result},
    ops::Index,
};

use self::{
    fmt::BitsDisplayExt,
    iter::{BitPackedExt, BitUnpackedExt, BitmapIntoIter, BitmapIter},
};
use crate::{
    buffer::{Buffer, BufferAlloc, BufferExtend, BufferMut, BufferRef, BufferRefMut, BufferTake},
    Length,
};

mod fmt;

pub mod iter;

mod validity;
pub use validity::ValidityBitmap;

/// A collection of bits.
///
/// The validity bits are stored LSB-first in the bytes of a [Buffer].
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Bitmap<BitmapBuffer = Vec<u8>>
// where
//     BitmapBuffer: Buffer<u8>,
{
    /// The bits are stored in this buffer.
    buffer: BitmapBuffer,

    /// The number of bits stored in the bitmap.
    bits: usize,

    /// An offset (in number of bits) in the buffer. This enables zero-copy
    /// slicing of the bitmap on non-byte boundaries.
    offset: usize,
}

impl<BitmapBuffer> Bitmap<BitmapBuffer>
where
    BitmapBuffer: Buffer<u8>,
{
    /// Forms a Bitmap from a [Buffer<u8>], a number of bits and an offset (in
    /// bits).
    ///
    /// # Safety
    ///
    /// Caller must ensure that the buffer contains enough bytes for the
    /// specified number of bits including the offset.
    #[cfg(feature = "unsafe")]
    pub unsafe fn from_raw_parts(buffer: BitmapBuffer, bits: usize, offset: usize) -> Self {
        Bitmap {
            buffer,
            bits,
            offset,
        }
    }

    /// Returns the bit at given bit index. Returns `None` when the index is out
    /// of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<bool> {
        (index < self.len()).then(||
            // Safety
            // - Bound checked
            unsafe { self.get_unchecked(index) })
    }

    /// Returns the bit at given bit index. Skips bound checking.
    ///
    /// # Safety
    ///
    /// Caller must ensure index is within bounds.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> bool {
        let (byte_index, bit_index) = self.index_pair(index);
        self.buffer.borrow().get_unchecked(byte_index) & 1 << bit_index != 0
    }
}

impl<BitmapBuffer> Bitmap<BitmapBuffer> {
    /// Returns the number of leading padding bits in the first byte(s) of the
    /// buffer that contain no meaningful bits. These bits should be ignored
    /// when inspecting the raw byte buffer.
    #[inline]
    pub fn leading_bits(&self) -> usize {
        self.offset
    }

    /// Returns the number of trailing padding bits in the last byte of the
    /// buffer that contain no meaningful bits. This bits should be ignored when
    /// inspecting the raw byte buffer.
    #[inline]
    pub fn trailing_bits(&self) -> usize {
        let trailing_bits = (self.offset + self.bits) % 8;
        if trailing_bits != 0 {
            8 - trailing_bits
        } else {
            0
        }
    }

    /// Returns the bit index for the element at the provided index.
    /// See [Bitmap::byte_index].
    #[inline]
    pub const fn bit_index(&self, index: usize) -> usize {
        (self.offset + index) % 8
    }

    /// Returns the byte index for the element at the provided index.
    /// See [Bitmap::bit_index].
    #[inline]
    pub const fn byte_index(&self, index: usize) -> usize {
        (self.offset + index) / 8
    }

    /// Returns the byte and bit index in the raw data buffer for the element at
    /// the provided index.
    #[inline]
    const fn index_pair(&self, index: usize) -> (usize, usize) {
        (self.byte_index(index), self.bit_index(index))
    }
}

impl<BitmapBuffer> ValidityBitmap for Bitmap<BitmapBuffer>
where
    BitmapBuffer: Buffer<u8>,
{
    type Buffer = BitmapBuffer;

    #[inline]
    fn validity_bitmap(&self) -> &Bitmap<BitmapBuffer> {
        self
    }
}

impl<BitmapBuffer> BufferRef for Bitmap<BitmapBuffer>
where
    BitmapBuffer: Buffer<u8>,
{
    type Buffer = BitmapBuffer;
    type Element = u8;

    fn buffer_ref(&self) -> &Self::Buffer {
        &self.buffer
    }
}

impl<BitmapBuffer> BufferRefMut for Bitmap<BitmapBuffer>
where
    BitmapBuffer: BufferMut<u8>,
{
    type BufferMut = BitmapBuffer;
    type Element = u8;

    fn buffer_ref_mut(&mut self) -> &mut Self::BufferMut {
        &mut self.buffer
    }
}

impl<BitmapBuffer> Debug for Bitmap<BitmapBuffer>
where
    BitmapBuffer: Buffer<u8>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct(&format!("Bitmap<{}>", any::type_name::<BitmapBuffer>()))
            .field("bits", &self.bits)
            .field("buffer", &format!("{}", self.buffer.bits_display()))
            .field("offset", &self.offset)
            .finish()
    }
}

impl<BitmapBuffer> Length for Bitmap<BitmapBuffer> {
    #[inline]
    fn len(&self) -> usize {
        self.bits
    }
}

impl<T, Buffer> Extend<T> for Bitmap<Buffer>
where
    T: Borrow<bool>,
    Buffer: BufferExtend<u8>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let mut additional_bits = 0;
        let mut iter = iter.into_iter().inspect(|_| {
            additional_bits += 1;
        });

        let trailing_bits = self.trailing_bits();
        if trailing_bits != 0 {
            let last_byte_index = self.byte_index(self.bits);
            let last_byte = &mut self.buffer.borrow_mut()[last_byte_index];
            for bit_position in 8 - trailing_bits..8 {
                if let Some(x) = iter.next() {
                    if *x.borrow() {
                        *last_byte |= 1 << bit_position;
                    }
                }
            }
        }

        self.buffer.extend(iter.bit_packed());
        self.bits += additional_bits;
    }
}

impl<T, BitmapBuffer> FromIterator<T> for Bitmap<BitmapBuffer>
where
    T: Borrow<bool>,
    BitmapBuffer: BufferAlloc<u8>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut bits = 0;
        let buffer = iter
            .into_iter()
            .inspect(|_| {
                bits += 1;
            })
            .bit_packed()
            .collect();

        Bitmap {
            bits,
            buffer,
            offset: 0,
        }
    }
}

impl<BitmapBuffer> Index<usize> for Bitmap<BitmapBuffer>
where
    BitmapBuffer: Buffer<u8>,
{
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("index (is {index}) should be < len (is {len})");
        }

        let len = self.bits;
        if index >= len {
            assert_failed(index, len);
        }

        // Safety:
        // - Bounds checked above.
        match unsafe { self.get_unchecked(index) } {
            true => &true,
            false => &false,
        }
    }
}

impl<'a, BitmapBuffer> IntoIterator for &'a Bitmap<BitmapBuffer>
where
    BitmapBuffer: Buffer<u8>,
{
    type IntoIter = BitmapIter<'a>;
    type Item = bool;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer
            .borrow()
            .iter()
            .bit_unpacked()
            .skip(self.offset)
            .take(self.bits)
    }
}

impl<BitmapBuffer> IntoIterator for Bitmap<BitmapBuffer>
where
    BitmapBuffer: BufferTake<u8>,
{
    type IntoIter = BitmapIntoIter<BitmapBuffer::IntoIter>;
    type Item = bool;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer
            .into_iter()
            .bit_unpacked()
            .skip(self.offset)
            .take(self.bits)
    }
}

impl<BitmapBuffer> PartialEq<[bool]> for Bitmap<BitmapBuffer>
where
    BitmapBuffer: Buffer<u8>,
{
    fn eq(&self, other: &[bool]) -> bool {
        self.len() == other.len()
            && self
                .into_iter()
                .zip(other.iter())
                .all(|(this, that)| this == *that)
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn bit_packed_size_hint() {
        assert_eq!((0, Some(0)), [].iter().bit_packed().size_hint());
        assert_eq!((1, Some(1)), [false].iter().bit_packed().size_hint());
        assert_eq!((1, Some(1)), [false; 7].iter().bit_packed().size_hint());
        assert_eq!((1, Some(1)), [false; 8].iter().bit_packed().size_hint());
        assert_eq!((2, Some(2)), [false; 9].iter().bit_packed().size_hint());
    }

    #[test]
    #[cfg(feature = "unsafe")]
    fn offset_byte_slice() {
        let mut bitmap = [true; 32].iter().collect::<Bitmap>();
        // "unset" first byte
        let slice = bitmap.buffer_ref_mut();
        slice[0] = 0;
        // "construct" new bitmap with last byte sliced off
        let bitmap_slice = unsafe { Bitmap::from_raw_parts(&slice[..3], 24, 0) };
        assert!(!bitmap_slice.all_valid());
    }

    #[test]
    #[cfg(feature = "unsafe")]
    fn offset_bit_slice() {
        let bitmap = unsafe { Bitmap::from_raw_parts([0b10100000u8], 3, 4) };
        assert_eq!(bitmap.len(), 3);
        assert_eq!(bitmap.leading_bits(), 4);
        assert_eq!(bitmap.trailing_bits(), 1);
        assert!(bitmap.is_null(0).unwrap());
        assert!(bitmap.is_valid(1).unwrap());
        assert!(bitmap.is_null(2).unwrap());
        assert_eq!(bitmap.null_count(), 2);
        assert_eq!(bitmap.valid_count(), 1);
        assert_eq!(bitmap.into_iter().collect::<Vec<_>>(), [false, true, false]);
    }

    #[test]
    #[cfg(feature = "unsafe")]
    fn offset_byte_vec() {
        let mut bitmap = [true; 32].iter().collect::<Bitmap<Vec<u8>>>();
        // "unset" first byte
        let vec: &mut Vec<u8> = bitmap.buffer_ref_mut();
        vec[0] = 0;
        // "construct" new bitmap with last byte sliced off
        let bitmap_sliced = unsafe { Bitmap::from_raw_parts(&vec[..3], 24, 0) };
        assert!(!bitmap_sliced.all_valid());
    }

    #[test]
    fn from_slice() {
        let bitmap = Bitmap {
            bits: 5,
            buffer: [22u8].as_slice(),
            offset: 0,
        };
        let slice: &[u8] = bitmap.buffer_ref();
        assert_eq!(&slice[0], &22);
    }

    #[test]
    fn as_ref() {
        let bitmap = [false, true, true, false, true].iter().collect::<Bitmap>();
        let slice: &[u8] = bitmap.buffer_ref();
        assert_eq!(&slice[0], &22);
    }

    #[test]
    fn as_ref_u8() {
        let bitmap = [false, true, false, true, false, true]
            .iter()
            .collect::<Bitmap>();
        let bytes = bitmap.buffer_ref().as_bytes();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 42);
    }

    #[test]
    #[should_panic]
    fn as_ref_u8_out_of_bounds() {
        let bitmap = [false, true, false, true, false, true]
            .iter()
            .collect::<Bitmap>();
        let bits: &[u8] = bitmap.buffer_ref();
        let _ = bits[std::mem::size_of::<usize>()];
    }

    #[test]
    fn as_ref_bitslice() {
        let bits = [
            false, true, false, true, false, true, false, false, false, true,
        ]
        .iter()
        .collect::<Bitmap>();
        assert_eq!(bits.len(), 10);
        assert!(!bits[0]);
        assert!(bits[1]);
        assert!(!bits[2]);
        assert!(bits[3]);
        assert!(!bits[4]);
        assert!(bits[5]);
        assert!(!bits[6]);
        assert!(!bits[7]);
        assert!(!bits[8]);
        assert!(bits[9]);
    }

    #[test]
    #[should_panic]
    fn as_ref_bitslice_out_of_bounds() {
        let bitmap = vec![false, true, false, true, false, true]
            .iter()
            .collect::<Bitmap>();
        let _ = bitmap[bitmap.bits];
    }

    #[test]
    fn count() {
        let vec = vec![false, true, false, true, false, true];
        let bitmap = vec.iter().collect::<Bitmap>();
        assert_eq!(bitmap.len(), 6);
        assert!(!bitmap.is_empty());
        assert_eq!(bitmap.valid_count(), 3);
        assert_eq!(bitmap.null_count(), 3);
        vec.iter()
            .zip(bitmap.into_iter())
            .for_each(|(a, b)| assert_eq!(*a, b));
    }

    #[test]
    fn from_iter() {
        let vec = vec![true, false, true, false];
        let bitmap = vec.iter().collect::<Bitmap>();
        assert_eq!(bitmap.len(), vec.len());
        assert_eq!(vec, bitmap.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn from_iter_ref() {
        let array = [true, false, true, false];
        let bitmap = array.iter().collect::<Bitmap>();
        assert_eq!(bitmap.len(), array.len());
        assert_eq!(array.to_vec(), bitmap.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn into_iter() {
        let vec = vec![true, false, true, false];
        let bitmap = vec.iter().collect::<Bitmap>();
        assert_eq!(bitmap.into_iter().collect::<Vec<_>>(), vec);
    }

    #[test]
    fn size_of() {
        assert_eq!(
            mem::size_of::<Bitmap>(),
            mem::size_of::<Vec<u8>>() + 2 * mem::size_of::<usize>()
        );

        assert_eq!(
            mem::size_of::<Bitmap<Box<[u8]>>>(),
            mem::size_of::<Box<[u8]>>() + 2 * mem::size_of::<usize>()
        );
    }
}
