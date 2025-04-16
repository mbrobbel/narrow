//! A collection of bits.

use crate::{
    Index, Length,
    buffer::{Buffer, BufferMut, BufferRef, BufferRefMut, BufferType, VecBuffer},
};
use std::{
    any,
    borrow::Borrow,
    fmt::{Debug, Formatter, Result},
    ops,
};

mod iter;
use self::iter::{BitPackedExt, BitUnpackedExt};
pub use self::iter::{BitmapIntoIter, BitmapIter};

mod fmt;
use self::fmt::BitsDisplayExt;

mod validity;
pub use self::validity::ValidityBitmap;

/// An immutable reference to a bitmap.
pub trait BitmapRef {
    /// The buffer type of the bitmap.
    type Buffer: BufferType;

    /// Returns a reference to an immutable [Bitmap].
    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer>;
}

/// A mutable reference to a bitmap.
pub trait BitmapRefMut: BitmapRef {
    /// Returns a mutable reference to a [Bitmap].
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer>;
}

/// A collection of bits.
///
/// The validity bits are stored LSB-first in the bytes of the `Buffer`.
// todo(mb): implement ops
pub struct Bitmap<Buffer: BufferType = VecBuffer> {
    /// The bits are stored in this buffer of bytes.
    pub(crate) buffer: <Buffer as BufferType>::Buffer<u8>,

    /// The number of bits stored in the bitmap.
    pub(crate) bits: usize,

    /// An offset (in number of bits) in the buffer. This enables zero-copy
    /// slicing of the bitmap on non-byte boundaries.
    pub(crate) offset: usize,
}

impl<Buffer: BufferType> BitmapRef for Bitmap<Buffer> {
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self
    }
}

impl<Buffer: BufferType> Bitmap<Buffer> {
    /// Returns an iterator over the bits in this [`Bitmap`].
    pub fn iter(&self) -> BitmapIter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }

    /// Forms a Bitmap from a buffer, a number of bits and an offset (in
    /// bits).
    ///
    /// # Safety
    ///
    /// Caller must ensure that the buffer contains enough bytes for the
    /// specified number of bits including the offset.
    pub unsafe fn from_raw_parts(
        buffer: <Buffer as BufferType>::Buffer<u8>,
        bits: usize,
        offset: usize,
    ) -> Self {
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
            // Safety:
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
        self.buffer.as_slice().get_unchecked(self.byte_index(index)) & (1 << self.bit_index(index))
            != 0
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
    #[inline]
    pub fn trailing_bits(&self) -> usize {
        let trailing_bits = self.bit_index(self.bits);
        if trailing_bits == 0 {
            0
        } else {
            8 - trailing_bits
        }
    }

    /// Returns the bit index for the element at the provided index.
    /// See [`Bitmap::byte_index`].
    #[inline]
    pub fn bit_index(&self, index: usize) -> usize {
        (self.offset + index) % 8
    }

    /// Returns the byte index for the element at the provided index.
    /// See [`Bitmap::bit_index`].
    #[inline]
    pub fn byte_index(&self, index: usize) -> usize {
        (self.offset + index) / 8
    }

    /// Returns a [`Bitmap`] with `len` bits set.
    #[must_use]
    pub fn new_valid(len: usize) -> Self
    where
        Self: FromIterator<bool>,
    {
        // TODO: optimize
        std::iter::repeat(true).take(len).collect::<Self>()
    }
}

impl<Buffer: BufferType> BufferRef<u8> for Bitmap<Buffer> {
    type Buffer = <Buffer as BufferType>::Buffer<u8>;

    fn buffer_ref(&self) -> &Self::Buffer {
        &self.buffer
    }
}

impl<Buffer> BufferRefMut<u8> for Bitmap<Buffer>
where
    Buffer: BufferType<Buffer<u8>: BufferMut<u8>>,
{
    type BufferMut = <Buffer as BufferType>::Buffer<u8>;

    fn buffer_ref_mut(&mut self) -> &mut Self::BufferMut {
        &mut self.buffer
    }
}

impl<Buffer> Clone for Bitmap<Buffer>
where
    Buffer: BufferType<Buffer<u8>: Clone>,
{
    fn clone(&self) -> Self {
        Bitmap {
            buffer: self.buffer.clone(),
            bits: self.bits,
            offset: self.offset,
        }
    }
}

impl<Buffer: BufferType> Debug for Bitmap<Buffer> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct(&format!("Bitmap<{}>", any::type_name::<Buffer>()))
            .field("bits", &self.bits)
            .field("buffer", &format!("{}", self.buffer.bits_display()))
            .field("offset", &self.offset)
            .finish()
    }
}

impl<Buffer> Default for Bitmap<Buffer>
where
    Buffer: BufferType<Buffer<u8>: Default>,
{
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            bits: Default::default(),
            offset: Default::default(),
        }
    }
}

impl<T, Buffer> Extend<T> for Bitmap<Buffer>
where
    T: Borrow<bool>,
    Buffer: BufferType<Buffer<u8>: BufferMut<u8> + Extend<u8>>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let mut additional_bits = 0;
        let mut items = iter.into_iter().inspect(|_| {
            additional_bits += 1;
        });

        let trailing_bits = self.trailing_bits();
        if trailing_bits != 0 {
            let last_byte_index = self.byte_index(self.bits);
            let last_byte = &mut self.buffer.as_mut_slice()[last_byte_index];
            for bit_position in 8 - trailing_bits..8 {
                if let Some(x) = items.next() {
                    if *x.borrow() {
                        *last_byte |= 1 << bit_position;
                    }
                }
            }
        }

        self.buffer.extend(items.bit_packed());
        self.bits += additional_bits;
    }
}

impl<Buffer, T> FromIterator<T> for Bitmap<Buffer>
where
    T: Borrow<bool>,
    Buffer: BufferType<Buffer<u8>: FromIterator<u8>>,
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
        Self {
            buffer,
            bits,
            offset: 0,
        }
    }
}

impl<Buffer: BufferType> Index for Bitmap<Buffer> {
    type Item<'a>
        = bool
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.get_unchecked(index)
    }
}

impl<Buffer: BufferType> ops::Index<usize> for Bitmap<Buffer> {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        /// Panic when out of bounds.
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
        if unsafe { self.get_unchecked(index) } {
            &true
        } else {
            &false
        }
    }
}

impl<'a, Buffer: BufferType> IntoIterator for &'a Bitmap<Buffer> {
    type Item = bool;
    type IntoIter = BitmapIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer
            .as_slice()
            .iter()
            .bit_unpacked()
            .skip(self.offset)
            .take(self.bits)
    }
}

impl<Buffer> IntoIterator for Bitmap<Buffer>
where
    Buffer: BufferType<Buffer<u8>: IntoIterator<Item = u8>>,
{
    type Item = bool;
    type IntoIter = BitmapIntoIter<<<Buffer as BufferType>::Buffer<u8> as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer
            .into_iter()
            .bit_unpacked()
            .skip(self.offset)
            .take(self.bits)
    }
}

impl<Buffer: BufferType> Length for Bitmap<Buffer> {
    fn len(&self) -> usize {
        self.bits
    }
}

impl<Buffer: BufferType> PartialEq for Bitmap<Buffer> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(a, b)| a == b)
    }
}

impl<const N: usize, Buffer: BufferType> PartialEq<[bool; N]> for Bitmap<Buffer> {
    fn eq(&self, other: &[bool; N]) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(a, b)| a == *b)
    }
}

impl<Buffer: BufferType> ValidityBitmap for Bitmap<Buffer> {}

#[cfg(test)]
mod tests {
    use crate::buffer::{ArrayBuffer, BoxBuffer, SliceBuffer};

    use super::*;
    use std::mem;

    #[test]
    fn offset_byte_slice() {
        let mut bitmap = [true; 32].iter().collect::<Bitmap>();
        // "unset" last byte
        let slice = bitmap.buffer_ref_mut();
        slice[3] = 0;
        // "construct" new bitmap with last byte sliced off
        // Safety:
        // - There are 24 bits in vec.
        let bitmap_sliced = unsafe { Bitmap::<SliceBuffer>::from_raw_parts(&slice[..3], 24, 0) };
        assert!(bitmap_sliced.all_valid());
    }

    #[test]
    fn offset_bit_slice() {
        use crate::buffer::ArrayBuffer;
        // Safety:
        // - 1 byte has 3 bits.
        let bitmap = unsafe { Bitmap::<ArrayBuffer<1>>::from_raw_parts([0b1010_0000], 3, 4) };
        assert_eq!(bitmap.len(), 3);
        assert_eq!(bitmap.leading_bits(), 4);
        assert_eq!(bitmap.trailing_bits(), 1);
        assert_eq!(bitmap.get(0), Some(false));
        assert_eq!(bitmap.get(1), Some(true));
        assert_eq!(bitmap.get(2), Some(false));
        assert_eq!((&bitmap).into_iter().filter(|x| !x).count(), 2);
        assert_eq!((&bitmap).into_iter().filter(|x| *x).count(), 1);
        assert_eq!(
            (&bitmap).into_iter().collect::<Vec<_>>(),
            [false, true, false]
        );
    }

    #[test]
    fn offset_byte_vec() {
        let mut bitmap = [true; 32].iter().collect::<Bitmap>();
        // "unset" last byte
        let vec: &mut Vec<u8> = bitmap.buffer_ref_mut();
        vec[3] = 0;
        // "construct" new bitmap with last byte sliced off
        // Safety:
        // - There are 24 bits in vec.
        let bitmap_sliced = unsafe { Bitmap::<SliceBuffer>::from_raw_parts(&vec[..3], 24, 0) };
        assert!(bitmap_sliced.all_valid());
    }

    #[test]
    fn from_slice() {
        let bitmap = Bitmap::<SliceBuffer> {
            bits: 5,
            buffer: &[42_u8],
            offset: 0,
        };
        let slice: &[u8] = bitmap.buffer_ref();
        assert_eq!(&slice[0], &42);
        let mut bitmap_22 = Bitmap::<ArrayBuffer<1>> {
            bits: 5,
            buffer: [22_u8],
            offset: 0,
        };
        let slice_22: &mut [u8] = bitmap_22.buffer_ref_mut();
        slice_22[0] += 20;
        assert_eq!(&slice_22[0], &42);
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
        let bytes = bitmap.buffer_ref();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 42);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
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
    #[should_panic(expected = "should be < len")]
    fn as_ref_bitslice_out_of_bounds() {
        let bitmap = [false, true, false, true, false, true]
            .iter()
            .collect::<Bitmap>();
        let _ = bitmap[bitmap.bits];
    }

    #[test]
    fn count() {
        let vec = [false, true, false, true, false, true];
        let bitmap = vec.iter().collect::<Bitmap>();
        assert_eq!(bitmap.len(), 6);
        assert!(!bitmap.is_empty());
        vec.iter().zip(bitmap).for_each(|(a, b)| assert_eq!(*a, b));
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
            mem::size_of::<Bitmap<BoxBuffer>>(),
            mem::size_of::<Box<[u8]>>() + 2 * mem::size_of::<usize>()
        );
    }
}
