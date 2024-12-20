//! Iterators for bitmaps.

use std::{
    iter::{Copied, Skip, Take},
    slice,
};

mod bit_packed;
pub use self::bit_packed::*;

mod bit_unpacked;
pub use self::bit_unpacked::*;

/// An iterator over the bits in a Bitmap.
///
/// This iterator returns boolean values that represent the bits stored in a
/// Bitmap.
pub type BitmapIter<'a> = Take<Skip<BitUnpacked<slice::Iter<'a, u8>, &'a u8>>>;

/// An iterator over the bits in a Bitmap. Consumes the Bitmap.
pub type BitmapIntoIter<I> = Copied<Take<Skip<BitUnpacked<I, u8>>>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_packing() {
        let input = [false, true, false, true, false, true];
        assert_eq!(
            input
                .iter()
                .bit_packed()
                .bit_unpacked()
                .take(input.len())
                .copied()
                .collect::<Vec<bool>>(),
            input
        );
    }

    #[test]
    fn additional_bits() {
        let input = [false, true];
        assert_eq!(
            input
                .iter()
                .bit_packed()
                .bit_unpacked()
                .copied()
                .collect::<Vec<bool>>(),
            [false, true, false, false, false, false, false, false]
        );
    }

    #[test]
    fn all_bits() {
        let input = [true, true, false, true, false, true, true, true];
        assert_eq!(
            input
                .iter()
                .bit_packed()
                .bit_unpacked()
                .copied()
                .collect::<Vec<bool>>(),
            input
        );
    }
}
