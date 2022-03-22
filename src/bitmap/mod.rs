//! A collection of bits.

use self::{
    fmt::BitsDisplayExt,
    iter::{BitPackedExt, BitUnpackedExt, BitmapIter},
};
use crate::{
    buffer::{Buffer, BufferAlloc, BufferExtend, BufferMut},
    DataBuffer, DataBufferMut, Length, Null,
};
use std::{
    any,
    fmt::{Debug, Formatter, Result},
    ops::Index,
};

pub mod fmt;
pub mod iter;

/// A collection of bits.
///
/// The validity bits are stored LSB-first in the bytes of a [Buffer].
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Bitmap<T = Box<[u8]>>
where
    T: Buffer<u8>,
{
    /// The number of bits stored in the bitmap.
    bits: usize,
    // todo(mb): offset - to slice bits without allocating
    /// The bits are stored in this buffer.
    buffer: T,
}

impl<T> Bitmap<T>
where
    T: Buffer<u8>,
{
    /// Forms a Bitmap from a [Buffer<u8>] and a number of bits.
    ///
    /// Safety:
    /// - Caller must ensure that the buffer contains at least the specified number of bits.
    pub unsafe fn from_raw_parts(buffer: T, bits: usize) -> Self {
        Bitmap { buffer, bits }
    }

    /// Returns the number of trailing padding bits in the last byte of the
    /// buffer that contain meaningful bits.
    pub fn trailing_bits(&self) -> usize {
        let trailing_bits = self.bits % 8;
        if trailing_bits != 0 {
            8 - trailing_bits
        } else {
            0
        }
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
            .finish()
    }
}

impl<T> Length for Bitmap<T>
where
    T: Buffer<u8>,
{
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
        let byte_index = index / 8;
        let bit_index = index % 8;
        self.buffer.borrow().get_unchecked(byte_index) & (1 << bit_index) != 0
    }

    fn valid_count(&self) -> usize {
        // Count all ones (there can't be ones in the padding bits).
        // > Bitmaps are to be initialized to be all unset at allocation time
        // > (this includes padding).
        self.buffer
            .borrow()
            .iter()
            .map(|x| x.count_ones())
            .sum::<u32>() as usize
    }

    fn null_count(&self) -> usize {
        // Count all zeros and subtract the trailing zero bits in the padding.
        // > Bitmaps are to be initialized to be all unset at allocation time
        // > (this includes padding).
        self.buffer
            .borrow()
            .iter()
            .map(|x| x.count_zeros())
            .sum::<u32>() as usize
            - self.trailing_bits()
    }
}

impl<T> Extend<bool> for Bitmap<T>
where
    T: BufferExtend<u8>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = bool>,
    {
        todo!()
    }

    #[cfg(feature = "extend_one")]
    fn extend_one(&mut self, item: bool) {
        self.extend(Some(item));
    }

    #[cfg(feature = "extend_one")]
    fn extend_reserve(&mut self, additional: usize) {
        self.buffer.extend_reserve(additional / 8)
    }
}

impl<T> FromIterator<bool> for Bitmap<T>
where
    T: BufferAlloc<u8>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = bool>,
    {
        let mut bits = 0;
        let buffer: T = iter
            .into_iter()
            .inspect(|_| {
                // Track the number of bits.
                bits += 1;
            })
            .bit_packed()
            .collect();
        Bitmap { bits, buffer }
    }
}

impl<'a, T> FromIterator<&'a bool> for Bitmap<T>
where
    T: BufferAlloc<u8>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a bool>,
    {
        let mut bits = 0;
        let buffer = iter
            .into_iter()
            .inspect(|_| {
                // Track the number of bits.
                bits += 1;
            })
            .bit_packed()
            .collect();
        Bitmap { bits, buffer }
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
        match unsafe { self.is_valid_unchecked(index) } {
            true => &true,
            false => &false,
        }
    }
}

impl<'a, T> IntoIterator for &'a Bitmap<T>
where
    T: Buffer<u8>,
{
    type Item = bool;
    type IntoIter = BitmapIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.borrow().iter().bit_unpacked().take(self.bits)
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
        let bitmap_slice = unsafe { Bitmap::from_raw_parts(&slice[..3], 24) };
        assert!(!bitmap_slice.all_valid());
    }

    #[test]
    fn offset_byte_vec() {
        let mut bitmap: Bitmap<Vec<u8>> = [true; 32].iter().collect();
        // "unset" first byte
        let vec: &mut Vec<u8> = bitmap.data_buffer_mut();
        vec[0] = 0;
        // "construct" new bitmap with last byte sliced off
        let bitmap_sliced = unsafe { Bitmap::from_raw_parts(&vec[..3], 24) };
        assert!(!bitmap_sliced.all_valid());
    }

    #[test]
    fn from_slice() {
        let bitmap = Bitmap {
            bits: 5,
            buffer: [22u8].as_slice(),
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
