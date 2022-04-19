//! A collection of bits.

use std::{
    any,
    borrow::Borrow,
    fmt::{Debug, Formatter, Result},
    ops::Index,
};

use self::{
    fmt::BitsDisplayExt,
    iter::{BitPackedExt, BitUnpackedExt, BitmapIter},
};
use crate::{
    buffer::{Buffer, BufferAlloc, BufferExtend, BufferMut},
    DataBuffer, DataBufferMut, Length, Null,
};

pub mod fmt;
pub mod iter;

/// A collection of bits.
///
/// The validity bits are stored LSB-first in the bytes of a [Buffer].
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Bitmap<T = Vec<u8>>
where
    T: Buffer<u8>,
{
    /// The bits are stored in this buffer.
    buffer: T,
    /// The number of bits stored in the bitmap.
    bits: usize,
    /// An offset (in number of bits) in the buffer. This enables zero-copy
    /// slicing of the bitmap on non-byte boundaries.
    offset: usize,
}

impl<T> Bitmap<T>
where
    T: Buffer<u8>,
{
    /// Forms a Bitmap from a [Buffer<u8>], a number of bits and an offset (in
    /// bits).
    ///
    /// # Safety
    ///
    /// Caller must ensure that the buffer contains enough bytes for the
    /// specified number of bits including the offset.
    pub unsafe fn from_raw_parts(buffer: T, bits: usize, offset: usize) -> Self {
        Bitmap {
            bits,
            buffer,
            offset,
        }
    }

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
    pub fn trailing_bits(&self) -> usize {
        let trailing_bits = (self.offset + self.bits) % 8;
        if trailing_bits != 0 {
            8 - trailing_bits
        } else {
            0
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

impl<T> DataBuffer<u8> for Bitmap<T>
where
    T: Buffer<u8>,
{
    type Buffer = T;

    fn data_buffer(&self) -> &Self::Buffer {
        &self.buffer
    }
}

impl<T> DataBufferMut<u8> for Bitmap<T>
where
    T: BufferMut<u8>,
{
    type Buffer = T;

    fn data_buffer_mut(&mut self) -> &mut Self::Buffer {
        &mut self.buffer
    }
}

impl<T> Debug for Bitmap<T>
where
    T: Buffer<u8>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct(&format!("Bitmap<{}>", any::type_name::<T>()))
            .field("bits", &self.bits)
            .field("buffer", &format!("{}", self.buffer.bits_display()))
            .field("offset", &self.offset)
            .finish()
    }
}

impl<T> Length for Bitmap<T>
where
    T: Buffer<u8>,
{
    #[inline]
    fn len(&self) -> usize {
        self.bits
    }
}

impl<T> Null for Bitmap<T>
where
    T: Buffer<u8>,
{
    #[inline]
    unsafe fn is_valid_unchecked(&self, index: usize) -> bool {
        self.get_unchecked(index)
    }

    // todo(mb): add optimized impls for other methods
}

impl<T, U> Extend<U> for Bitmap<T>
where
    T: BufferExtend<u8>,
    U: Borrow<bool>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = U>,
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

    #[cfg(feature = "extend_one")]
    fn extend_one(&mut self, item: U) {
        self.extend(Some(item));
    }

    #[cfg(feature = "extend_one")]
    fn extend_reserve(&mut self, additional: usize) {
        // Only reserve when the additional bits do not fit in the trailing bits.
        if additional > self.trailing_bits() {
            self.buffer.extend_reserve(additional / 8)
        }
    }
}

impl<T, U> FromIterator<U> for Bitmap<T>
where
    T: BufferAlloc<u8>,
    U: Borrow<bool>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = U>,
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

impl<T> Index<usize> for Bitmap<T>
where
    T: Buffer<u8>,
{
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("index (is {}) should be < len (is {})", index, len);
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

impl<'a, T> IntoIterator for &'a Bitmap<T>
where
    T: Buffer<u8>,
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

#[cfg(test)]
mod tests {
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
    fn offset_byte_slice() {
        let mut bitmap: Bitmap = [true; 32].iter().collect();
        // "unset" first byte
        let slice = bitmap.data_buffer_mut();
        slice[0] = 0;
        // "construct" new bitmap with last byte sliced off
        let bitmap_slice = unsafe { Bitmap::from_raw_parts(&slice[..3], 24, 0) };
        assert!(!bitmap_slice.all_valid());
    }

    #[test]
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
    fn offset_byte_vec() {
        let mut bitmap: Bitmap<Vec<u8>> = [true; 32].iter().collect();
        // "unset" first byte
        let vec: &mut Vec<u8> = bitmap.data_buffer_mut();
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
        let slice: &[u8] = bitmap.data_buffer().as_ref();
        assert_eq!(&slice[0], &22);
    }

    #[test]
    fn as_ref() {
        let bitmap: Bitmap = [false, true, true, false, true].iter().collect();
        let slice: &[u8] = bitmap.data_buffer().as_ref();
        assert_eq!(&slice[0], &22);
    }

    #[test]
    fn as_ref_u8() {
        let bitmap: Bitmap = [false, true, false, true, false, true].iter().collect();
        let bytes = bitmap.data_buffer().as_bytes();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 42);
    }

    #[test]
    #[should_panic]
    fn as_ref_u8_out_of_bounds() {
        let bitmap: Bitmap = [false, true, false, true, false, true].iter().collect();
        let bits: &[u8] = bitmap.data_buffer().as_ref();
        let _ = bits[std::mem::size_of::<usize>()];
    }

    #[test]
    fn as_ref_bitslice() {
        let bits: Bitmap = [
            false, true, false, true, false, true, false, false, false, true,
        ]
        .iter()
        .collect();
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
        let bitmap: Bitmap = vec![false, true, false, true, false, true].iter().collect();
        let _ = bitmap[bitmap.bits];
    }

    #[test]
    fn count() {
        let vec = vec![false, true, false, true, false, true];
        let bitmap: Bitmap = vec.iter().collect();
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
        let bitmap: Bitmap = vec.iter().collect();
        assert_eq!(bitmap.into_iter().collect::<Vec<_>>(), vec);
    }
}
