use crate::{buffer, ArrayData, ArrayIndex, Buffer, ALIGNMENT};
use bitvec::{
    order::Lsb0,
    slice::{BitSlice, Iter}, //, BitValIter},
    view::BitView,
};
use std::{
    iter::{Copied, FromIterator},
    ops::Deref,
};

/// An immutable collection of bits.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bitmap {
    /// The number of bits stored in the bitmap.
    bits: usize,
    /// The bits are stored in the buffer.
    buffer: Buffer<usize, ALIGNMENT>,
}

impl ArrayIndex<usize> for Bitmap {
    type Output = bool;

    fn index(&self, index: usize) -> Self::Output {
        self.is_valid(index)
    }
}

impl ArrayData for Bitmap {
    fn len(&self) -> usize {
        self.bits
    }

    fn is_null(&self, index: usize) -> bool {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("is_null index (is {}) should be < len (is {})", index, len);
        }

        let len = self.len();
        if index >= len {
            assert_failed(index, len);
        }

        let slice: &BitSlice<_, _> = self.as_ref();
        // Safety:
        // - Bounds checked above
        unsafe { !slice.get_unchecked(index) }
    }

    fn null_count(&self) -> usize {
        0
    }

    fn is_valid(&self, index: usize) -> bool {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("is_valid index (is {}) should be < len (is {})", index, len);
        }

        let len = self.len();
        if index >= len {
            assert_failed(index, len);
        }

        let slice: &BitSlice<_, _> = self.as_ref();
        // Safety:
        // - Bounds checked above
        unsafe { *slice.get_unchecked(index) }
    }

    fn valid_count(&self) -> usize {
        self.bits
    }
}

impl AsRef<[u8]> for Bitmap {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

impl AsRef<[usize]> for Bitmap {
    fn as_ref(&self) -> &[usize] {
        &self.buffer[..]
    }
}

impl AsRef<Bitmap> for Bitmap {
    fn as_ref(&self) -> &Bitmap {
        self
    }
}

impl AsRef<BitSlice<Lsb0, usize>> for Bitmap {
    fn as_ref(&self) -> &BitSlice<Lsb0, usize> {
        self
    }
}

impl Deref for Bitmap {
    type Target = BitSlice<Lsb0, usize>;

    fn deref(&self) -> &Self::Target {
        // Safety
        // - Number of bits is an invariant of bitmap.
        unsafe { self.buffer.view_bits::<Lsb0>().get_unchecked(..self.bits) }
    }
}

impl FromIterator<bool> for Bitmap {
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

                // Get the number of words required to store this many bits.
                const WIDTH: usize = usize::BITS as usize;
                let mut len = bits / WIDTH + (bits % WIDTH != 0) as usize;

                // Allocate memory for the storage of the words.
                let mut ptr = unsafe {
                    buffer::alloc::<usize, ALIGNMENT>(buffer::layout::<usize, ALIGNMENT>(len))
                };

                // Single word that is written to the buffer when its bits are
                // set according to the input.
                let mut word = if value { 1 } else { 0 };

                // Word index counter. To track the current position
                let mut word_index = 0;

                // Bit mask to set the bit. This starts at 2 because the first
                // bit is already set in the word according to the first value
                // returned by the iterator.
                let mut mask = 2usize;

                // Count the total number of bits.
                let mut bits = 1;

                for bit in iter {
                    if bit {
                        // Set bit in word using mask as position.
                        word |= mask;
                    }

                    // Update mask for next bit.
                    mask = mask.rotate_left(1);

                    // When the mask wraps the next item goes to the next word.
                    // The current word is written to the current word index.
                    if mask == 1 {
                        // Check capacity.
                        if word_index == len {
                            // Make sure an additional word can be written to the
                            // buffer.
                            ptr = unsafe {
                                buffer::realloc::<usize, ALIGNMENT, ALIGNMENT>(ptr, len, len + 1)
                            };
                            len += 1;
                        }

                        // Write the word.
                        unsafe { ptr.add(word_index).write(word) };

                        // Reset word
                        word = 0;

                        // Point to next word in buffer.
                        word_index += 1;
                    }

                    // Count number of bits.
                    bits += 1;
                }

                // Write last word (when required).
                if mask != 1 {
                    // Check capacity
                    if word_index == len {
                        // Make sure an additional word can be written to the
                        // buffer.
                        ptr = unsafe {
                            buffer::realloc::<usize, ALIGNMENT, ALIGNMENT>(ptr, len, len + 1)
                        };
                        len += 1;
                    }

                    unsafe { ptr.add(word_index).write(word) };
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

/// Iterator over bits in a bitmap.
pub type BitmapIter<'a> = Copied<Iter<'a, Lsb0, usize>>; //BitValIter<'a, Lsb0, usize>;

impl<'a> IntoIterator for &'a Bitmap {
    type Item = bool;
    type IntoIter = BitmapIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter().copied() //by_val()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn capacity() {
        let vec = vec![true; usize::BITS as usize - 1];
        let bitmap: Bitmap = vec.iter().copied().collect();
        let words: &[usize] = bitmap.as_ref();
        assert_eq!(words.len(), 1);

        let vec = vec![true; usize::BITS as usize];
        let bitmap: Bitmap = vec.iter().copied().collect();
        let words: &[usize] = bitmap.as_ref();
        assert_eq!(words.len(), 1);

        let vec = vec![true; usize::BITS as usize + 1];
        let bitmap: Bitmap = vec.iter().copied().collect();
        let words: &[usize] = bitmap.as_ref();
        assert_eq!(words.len(), 2);
    }

    #[test]
    fn as_ref() {
        let bitmap: Bitmap = [false, true, true, false, true].iter().copied().collect();
        let slice: &[u8] = bitmap.as_ref();
        assert_eq!(&slice[0], &22);
        assert_eq!(&bitmap, bitmap.as_ref());
    }

    #[test]
    fn as_ref_u8() {
        let bitmap: Bitmap = vec![false, true, false, true, false, true]
            .iter()
            .copied()
            .collect();
        let bytes: &[u8] = bitmap.as_ref();
        assert_eq!(bytes.len(), mem::size_of::<usize>());
        assert_eq!(bytes[0], 42);
        assert_eq!(bytes[1..], [0; mem::size_of::<usize>() - 1]);
    }

    #[test]
    #[should_panic]
    fn as_ref_u8_out_of_bounds() {
        let bitmap: Bitmap = vec![false, true, false, true, false, true]
            .iter()
            .copied()
            .collect();
        let bits: &[u8] = bitmap.as_ref();
        let _ = bits[mem::size_of::<usize>()];
    }

    #[test]
    fn as_ref_usize() {
        let bitmap: Bitmap = vec![false, true, false, true, false, true]
            .iter()
            .copied()
            .collect();
        let words: &[usize] = bitmap.as_ref();
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], 42);
    }

    #[test]
    fn as_ref_bitslice() {
        let bitmap: Bitmap = vec![false, true, false, true, false, true]
            .iter()
            .copied()
            .collect();
        let bits: &BitSlice<_, _> = bitmap.as_ref();
        assert_eq!(bits.len(), 6);
        assert!(!bits[0]);
        assert!(bits[1]);
        assert!(!bits[2]);
        assert!(bits[3]);
        assert!(!bits[4]);
        assert!(bits[5]);
    }

    #[test]
    #[should_panic]
    fn as_ref_bitslice_out_of_bounds() {
        let bitmap: Bitmap = vec![false, true, false, true, false, true]
            .iter()
            .copied()
            .collect();
        let bits: &BitSlice<_, _> = bitmap.as_ref();
        let _ = bits[bits.len()];
    }

    #[test]
    fn deref() {
        let vec = vec![false, true, false, true, false, true];
        let bitmap: Bitmap = vec.iter().copied().collect();
        assert_eq!(bitmap.len(), 6);
        assert!(!bitmap.is_empty());
        assert_eq!(bitmap.count_ones(), 3);
        assert_eq!(bitmap.count_zeros(), 3);
        vec.iter()
            // .zip(bitmap.iter().by_val())
            .zip(bitmap.iter().copied())
            .for_each(|(a, b)| assert_eq!(*a, b));
        assert_eq!(bitmap.buffer.as_ptr(), bitmap.as_raw_slice().as_ptr());
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
