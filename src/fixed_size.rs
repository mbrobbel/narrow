//! Subtrait for fixed-size types.

use crate::array::ArrayType;
use std::{fmt::Debug, mem};

/// Subtrait for fixed-size types.
///
/// This exists to be used as trait bound where one or more of the supertraits
/// of this trait are required, and to restrict certain implementations to
/// fixed-size types.
///
/// This trait is sealed to prevent downstream implementations.
pub trait FixedSize: ArrayType + Copy + Debug + Sized + sealed::Sealed + 'static {
    /// The fixed-size of this type in bytes.
    const SIZE: usize = mem::size_of::<Self>();
}

impl FixedSize for i8 {}
impl FixedSize for i16 {}
impl FixedSize for i32 {}
impl FixedSize for i64 {}
impl FixedSize for i128 {}
impl FixedSize for u8 {}
impl FixedSize for u16 {}
impl FixedSize for u32 {}
impl FixedSize for u64 {}
impl FixedSize for u128 {}

impl FixedSize for isize {}
impl FixedSize for usize {}

impl FixedSize for f32 {}
impl FixedSize for f64 {}

impl FixedSize for () {}

impl<const N: usize, T: FixedSize> FixedSize for [T; N] {}

mod sealed {
    /// Used to seal [super::FixedSize].
    pub trait Sealed {}

    // Prevent downstream implementation of [super::FixedSize].
    impl<T> Sealed for T where T: super::FixedSize {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size() {
        assert_eq!(<()>::SIZE, 0);
        assert_eq!(u8::SIZE, 1);
        assert_eq!(<[u16; 21]>::SIZE, 42);
        assert_eq!(<[u8; 1234]>::SIZE, 1234);
    }
}
