use crate::buffer::Buffer;
use std::fmt::{Display, Formatter, Result};

/// A slice wrapper with a [Display] implementation to format bytes as bits.
pub(crate) struct BitsDisplay<'a>(&'a [u8]);

impl Display for BitsDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_list()
            .entries(self.0.iter().map(|byte| format!("{:08b}", byte)))
            .finish()
    }
}

pub(crate) trait BitsDisplayExt {
    fn bits_display(&self) -> BitsDisplay<'_>;
}

impl<T> BitsDisplayExt for T
where
    T: Buffer<u8>,
{
    fn bits_display(&self) -> BitsDisplay<'_> {
        BitsDisplay(self.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_display() {
        assert_eq!(
            format!("{}", &[1u8, 2, 3, 4, u8::MAX].bits_display()),
            "[\"00000001\", \"00000010\", \"00000011\", \"00000100\", \"11111111\"]"
        );
    }
}
