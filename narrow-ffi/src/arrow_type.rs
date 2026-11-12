//! Arrow C Data format strings of Rust item types.

use core::ffi::CStr;

use crate::ARROW_FLAG_NULLABLE;

/// A type with an [Arrow C Data format string].
///
/// [Arrow C Data format string]: https://arrow.apache.org/docs/format/CDataInterface.html#data-type-description-format-strings
///
/// The Rust item type already determines its Arrow layout. These associated
/// constants carry the corresponding C schema description at the type level,
/// including the nullable flag added by `Option<T>`.
///
/// # Examples
///
/// ```
/// use narrow_ffi::{ARROW_FLAG_NULLABLE, ArrowType};
///
/// assert_eq!(i32::FORMAT, c"i");
/// assert_eq!(<Option<i32>>::FLAGS, ARROW_FLAG_NULLABLE);
/// ```
pub trait ArrowType {
    /// Arrow C Data type format.
    const FORMAT: &'static CStr;
    /// Arrow C Data schema flags.
    const FLAGS: i64 = 0;
}

impl<T: ArrowType> ArrowType for Option<T> {
    const FORMAT: &'static CStr = T::FORMAT;
    const FLAGS: i64 = T::FLAGS | ARROW_FLAG_NULLABLE;
}

impl ArrowType for bool {
    const FORMAT: &'static CStr = c"b";
}

impl ArrowType for i8 {
    const FORMAT: &'static CStr = c"c";
}

impl ArrowType for u8 {
    const FORMAT: &'static CStr = c"C";
}

impl ArrowType for i16 {
    const FORMAT: &'static CStr = c"s";
}

impl ArrowType for u16 {
    const FORMAT: &'static CStr = c"S";
}

impl ArrowType for i32 {
    const FORMAT: &'static CStr = c"i";
}

impl ArrowType for u32 {
    const FORMAT: &'static CStr = c"I";
}

impl ArrowType for i64 {
    const FORMAT: &'static CStr = c"l";
}

impl ArrowType for u64 {
    const FORMAT: &'static CStr = c"L";
}

impl ArrowType for f32 {
    const FORMAT: &'static CStr = c"f";
}

impl ArrowType for f64 {
    const FORMAT: &'static CStr = c"g";
}

#[cfg(test)]
mod tests {
    use crate::ARROW_FLAG_NULLABLE;

    use super::ArrowType;

    #[test]
    fn type_format_strings_match_arrow() {
        assert_eq!(i8::FORMAT, c"c");
        assert_eq!(u8::FORMAT, c"C");
        assert_eq!(i16::FORMAT, c"s");
        assert_eq!(u16::FORMAT, c"S");
        assert_eq!(i32::FORMAT, c"i");
        assert_eq!(u32::FORMAT, c"I");
        assert_eq!(i64::FORMAT, c"l");
        assert_eq!(u64::FORMAT, c"L");
        assert_eq!(f32::FORMAT, c"f");
        assert_eq!(f64::FORMAT, c"g");
        assert_eq!(<Option<i32>>::FORMAT, c"i");
        assert_eq!(<Option<i32>>::FLAGS, ARROW_FLAG_NULLABLE);
    }

    #[test]
    fn type_format_string_matches_arrow() {
        assert_eq!(bool::FORMAT, c"b");
    }
}
