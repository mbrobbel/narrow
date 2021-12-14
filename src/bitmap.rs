use crate::{Buffer, Length, Null, DEFAULT_ALIGNMENT};
use std::{ops::Index, slice::Iter};

/// An immutable collection of bits.
///
/// The validity bits are stored LSB-first in the bytes of a [Buffer].
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Bitmap<const A: usize = DEFAULT_ALIGNMENT> {
    /// The number of bits stored in the bitmap.
    bits: usize,
    /// The bits are stored in this buffer.
    buffer: Buffer<u8, A>,
}

impl<const A: usize> Null for Bitmap<A> {
    unsafe fn is_valid_unchecked(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let bit_index = index % 8;
        self.buffer.get_unchecked(byte_index) & (1 << bit_index) != 0
    }

    fn valid_count(&self) -> usize {
        // Count all ones (there can't be ones in the padding bits).
        // > Bitmaps are to be initialized to be all unset at allocation time
        // > (this includes padding).
        self.buffer.iter().map(|x| x.count_ones()).sum::<u32>() as usize
    }

    fn null_count(&self) -> usize {
        // Count all zeros and subtract the trailing zero bits in the padding.
        // > Bitmaps are to be initialized to be all unset at allocation time
        // > (this includes padding).
        self.buffer.iter().map(|x| x.count_zeros()).sum::<u32>() as usize - self.trailing_bits()
    }
}

impl<const A: usize> Bitmap<A> {
    /// Returns the number of trailing padding bits in the last byte of the
    /// buffer that contains meaningful bits.
    pub fn trailing_bits(&self) -> usize {
        let trailing_bits = self.bits % 8;
        if trailing_bits != 0 {
            8 - trailing_bits
        } else {
            0
        }
    }
}

// todo(mb): use buffer traits here?
// impl<const A: usize> AsRef<[u8]> for Bitmap<A> {
//     fn as_ref(&self) -> &[u8] {
//         self.buffer.as_ref()
//     }
// }

impl<const A: usize> FromIterator<bool> for Bitmap<A> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = bool>,
    {
        let mut iter = iter.into_iter();

        // Check if the iterator is empty. If the iterator is empty an empty
        // bitmap is returned to prevent issues with zero-sized allocations.
        match iter.next() {
            Some(value) => {
                // Use the size hint to pre-allocate the buffer.
                let (lower_bound, _) = iter.size_hint();

                // We already advanced the iterator so add one to get the
                // expected number of bits.
                let bits = lower_bound + 1;

                // Get the number of bytes required to store this many bits.
                let mut len = bits / 8 + (bits % 8 != 0) as usize;

                // Allocate memory for the storage of the bytes.
                let mut ptr = unsafe { Buffer::<u8, A>::alloc(len) };

                // Single byte that is written to the buffer when its bits are
                // set according to the input.
                let mut byte = if value { 1 } else { 0 };

                // Byte index counter. To track the current position
                let mut byte_index = 0;

                // Bit mask to set the bit. This starts at 2 because the first
                // bit is already set in the word according to the first value
                // returned by the iterator.
                let mut mask = 2u8;

                // Count the total number of bits.
                let mut bits = 1;

                for bit in iter {
                    if bit {
                        // Set bit in byte using mask as position.
                        byte |= mask;
                    }

                    // Update mask for next bit.
                    mask = mask.rotate_left(1);

                    // When the mask wraps the next item goes to the next byte.
                    // The current byte is written to the current byte index.
                    if mask == 1 {
                        // Check capacity.
                        if byte_index == len {
                            // Make sure an additional byte can be written to the
                            // buffer.
                            ptr = unsafe { Buffer::<u8, A>::realloc(ptr, len, len + 1) };
                            len += 1;
                        }

                        // Write the byte.
                        unsafe { ptr.add(byte_index).write(byte) };

                        // Reset byte.
                        byte = 0;

                        // Point to next byte in buffer.
                        byte_index += 1;
                    }

                    // Count number of bits.
                    bits += 1;
                }

                // Write last byte (when required).
                if mask != 1 {
                    // Check capacity
                    if byte_index == len {
                        // Make sure an additional byte can be written to the
                        // buffer.
                        ptr = unsafe { Buffer::<u8, A>::realloc(ptr, len, len + 1) };
                        len += 1;
                    }

                    unsafe { ptr.add(byte_index).write(byte) };
                }

                Self {
                    bits,
                    buffer: unsafe { Buffer::new_unchecked(ptr, len) },
                }
            }
            None => Self::default(),
        }
    }
}

impl<const A: usize> Index<usize> for Bitmap<A> {
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

impl<'a, const A: usize> IntoIterator for &'a Bitmap<A> {
    type Item = bool;
    type IntoIter = BitmapIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BitmapIter {
            inner: self.buffer.iter(),
            byte: None,
            // mask rotates left at start of iteration and selects first bit at
            // first iteration.
            mask: 1 << 7,
            remaining: self.bits,
        }
    }
}

impl<const A: usize> Length for Bitmap<A> {
    fn len(&self) -> usize {
        self.bits
    }
}

/// Iterator over the bits of a Bitmap.
pub struct BitmapIter<'a> {
    // Iterator over the bytes stored in the buffer of the bitmap.
    inner: Iter<'a, u8>,
    // Last byte popped from inner iterator.
    byte: Option<&'a u8>,
    // Mask to select bits from the byte.
    mask: u8,
    // Keeps track of the number of bits that this iterator should produce.
    remaining: usize,
}

impl<'a> Iterator for BitmapIter<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        // Return if we reached the end
        if self.remaining == 0 {
            return None;
        }

        // Update mask for next bit.
        self.mask = self.mask.rotate_left(1);

        // Fetch next byte when mask wraps around
        if self.mask == 1 {
            self.byte = self.inner.next();
        }

        // Check if bit at current index is set
        let next = self.byte.map(|byte| byte & self.mask != 0);

        // Update remaining bits info.
        self.remaining -= 1;

        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn capacity() {
    //     let vec = vec![true; u8::BITS as usize - 1];
    //     let bitmap: Bitmap = vec.iter().copied().collect();
    //     let bytes: &[u8] = bitmap.as_ref();
    //     assert_eq!(bytes.len(), 1);

    //     let vec = vec![true; u8::BITS as usize];
    //     let bitmap: Bitmap = vec.iter().copied().collect();
    //     let bytes: &[u8] = bitmap.as_ref();
    //     assert_eq!(bytes.len(), 1);

    //     let vec = vec![true; u8::BITS as usize + 1];
    //     let bitmap: Bitmap = vec.iter().copied().collect();
    //     let bytes: &[u8] = bitmap.as_ref();
    //     assert_eq!(bytes.len(), 2);
    // }

    // #[test]
    // fn as_ref() {
    //     let bitmap: Bitmap = [false, true, true, false, true].iter().copied().collect();
    //     let slice: &[u8] = bitmap.as_ref();
    //     assert_eq!(&slice[0], &22);
    // }

    // #[test]
    // fn as_ref_u8() {
    //     let bitmap: Bitmap = vec![false, true, false, true, false, true]
    //         .iter()
    //         .copied()
    //         .collect();
    //     let bytes: &[u8] = bitmap.as_ref();
    //     assert_eq!(bytes.len(), mem::size_of::<u8>());
    //     assert_eq!(bytes[0], 42);
    //     assert_eq!(bytes[1..], [0; mem::size_of::<u8>() - 1]);
    // }

    // #[test]
    // #[should_panic]
    // fn as_ref_u8_out_of_bounds() {
    //     let bitmap: Bitmap = vec![false, true, false, true, false, true]
    //         .iter()
    //         .copied()
    //         .collect();
    //     let bits: &[u8] = bitmap.as_ref();
    //     let _ = bits[mem::size_of::<usize>()];
    // }

    #[test]
    fn as_ref_bitslice() {
        let bits: Bitmap = [
            false, true, false, true, false, true, false, false, false, true,
        ]
        .into_iter()
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
        let bitmap: Bitmap = vec![false, true, false, true, false, true]
            .iter()
            .copied()
            .collect();
        let _ = bitmap[bitmap.bits];
    }

    #[test]
    fn count() {
        let vec = vec![false, true, false, true, false, true];
        let bitmap: Bitmap = vec.iter().copied().collect();
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
        let bitmap = vec.iter().copied().collect::<Bitmap>();
        assert_eq!(bitmap.len(), vec.len());
        assert_eq!(vec, bitmap.into_iter().collect::<Vec<_>>());

        struct Foo {
            count: usize,
        }

        impl Iterator for Foo {
            type Item = bool;

            fn next(&mut self) -> Option<Self::Item> {
                if self.count != 0 {
                    self.count -= 1;
                    Some(true)
                } else {
                    None
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (0, None)
            }
        }

        let x = Foo { count: 1234 };
        let bitmap: Bitmap = x.into_iter().collect();
        assert_eq!(bitmap.len(), 1234);
    }

    #[test]
    fn from_iter_ref() {
        let array = [true, false, true, false];
        let bitmap = array.iter().copied().collect::<Bitmap>();
        assert_eq!(bitmap.len(), array.len());
        assert_eq!(array.to_vec(), bitmap.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn into_iter() {
        let vec = vec![true, false, true, false];
        let bitmap: Bitmap = vec.iter().copied().collect();
        assert_eq!(bitmap.into_iter().collect::<Vec<_>>(), vec);
    }
}
