//! Bitmap formatting.

use crate::buffer::Buffer;
use std::fmt::{Display, Formatter, Result};

/// A slice wrapper with a [Display] implementation to format bytes as bits.
pub(super) struct BitsDisplay<'a>(&'a [u8]);

impl Display for BitsDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_list()
            .entries(self.0.iter().map(|byte| format!("{byte:08b}")))
            .finish()
    }
}

/// Display a buffer of bytes as bits.
pub(super) trait BitsDisplayExt {
    /// Returns a wrapper around a buffer of bytes that implements `Display`.
    fn bits_display(&self) -> BitsDisplay<'_>;
}

impl<T> BitsDisplayExt for T
where
    T: Buffer<u8>,
{
    fn bits_display(&self) -> BitsDisplay<'_> {
        BitsDisplay(self.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_display() {
        assert_eq!(
            format!("{}", &[1, 2, 3, 4, u8::MAX].bits_display()),
            "[\"00000001\", \"00000010\", \"00000011\", \"00000100\", \"11111111\"]"
        );
    }
}
