//! # narrow
//!
//! A Rust implementation of Apache Arrow.

use std::ops::{Add, AddAssign, Sub};

// Hidden re-exports of types used in the `narrow-derive` crate.
#[doc(hidden)]
pub mod re_exports {
    extern crate self as narrow;
}

pub use narrow_derive::*;

/// Subtrait for primitive types.
///
/// This exists to use as trait bound where one or more of the supertraits of
/// this trait are required, and to restrict certain implementations to Arrow
/// primitive types.
pub trait Primitive:
    Add<Output = Self> + AddAssign + Copy + Default + Sub<Output = Self> + sealed::SealedPrimitive
{
}

impl Primitive for i8 {}
impl Primitive for i16 {}
impl Primitive for i32 {}
impl Primitive for i64 {}
impl Primitive for u8 {}
impl Primitive for u16 {}
impl Primitive for u32 {}
impl Primitive for u64 {}
impl Primitive for f32 {}
impl Primitive for f64 {}

// Sealed traits.
mod sealed {
    pub trait SealedPrimitive {}
    impl<T> SealedPrimitive for T where T: crate::Primitive {}
}
