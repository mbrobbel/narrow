//! Subtrait for fixed-size types.

use crate::array::ArrayType;
use std::{fmt::Debug, mem};

#[cfg(feature = "arrow-rs")]
/// Module that re-exports the [`arrow_buffer::ArrowNativeType`] trait.
mod arrow_rs {
    pub use arrow_buffer::ArrowNativeType as _arrow_rs_trait;
}
#[cfg(not(feature = "arrow-rs"))]
/// Module with empty trait to work around [RFC-3399](https://rust-lang.github.io/rfcs/3399-cfg-attribute-in-where.html).
mod arrow_rs {
    /// Empty trait.
    pub trait Type {}
    impl<T> Type for T {}
    pub use Type as _arrow_rs_trait;
}
use arrow_rs::_arrow_rs_trait;

/// Subtrait for fixed-size types.
///
/// This exists to be used as trait bound where one or more of the supertraits
/// of this trait are required, and to restrict certain implementations to
/// fixed-size types.
///
/// This trait is sealed to prevent downstream implementations.
pub trait FixedSize:
    ArrayType + Copy + Debug + Sized + sealed::Sealed + 'static + _arrow_rs_trait
{
    /// The fixed-size of this type in bytes.
    const SIZE: usize = mem::size_of::<Self>();
}

/// Private module for [`sealed::Sealed`] trait.
mod sealed {
    /// Used to seal [`super::FixedSize`].
    pub trait Sealed {}

    // Prevent downstream implementation of [super::FixedSize].
    impl<T> Sealed for T where T: super::FixedSize {}
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
#[cfg(not(feature = "arrow-rs"))]
impl FixedSize for u128 {}

#[cfg(not(feature = "arrow-rs"))]
impl FixedSize for isize {}
#[cfg(not(feature = "arrow-rs"))]
impl FixedSize for usize {}

impl FixedSize for f32 {}
impl FixedSize for f64 {}

#[cfg(not(feature = "arrow-rs"))]
impl<const N: usize, T: super::FixedSize> FixedSize for [T; N] {}

#[cfg(test)]
mod tests {
    use super::FixedSize;

    #[test]
    fn size() {
        assert_eq!(u8::SIZE, 1);
        #[cfg(not(feature = "arrow-rs"))]
        assert_eq!(<[u16; 21]>::SIZE, 42);
        #[cfg(not(feature = "arrow-rs"))]
        assert_eq!(<[u8; 1234]>::SIZE, 1234);
    }
}
