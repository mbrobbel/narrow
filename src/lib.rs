//! # narrow
//!
//! A Rust implementation of [Apache Arrow](https://arrow.apache.org).

use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Sub},
};

mod buffer;
pub use buffer::*;

mod bitmap;
pub use bitmap::*;

mod nullable;
pub use nullable::*;

mod validity;
pub use validity::*;

mod offset;
pub use offset::*;

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
///
/// This trait is sealed to prevent downstream implementations.
pub trait Primitive:
    Add<Output = Self>
    + AddAssign
    + Copy
    + Debug
    + Default
    + Sub<Output = Self>
    + sealed::SealedPrimitive
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

/// Types storing Arrow data.
///
/// This is implemented by [Buffer], [Bitmap] and [Nullable].
///
/// This trait is sealed to prevent downstream implementations.
pub trait Data: Default + sealed::SealedData {
    /// Returns the number elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Buffer, Data};
    ///
    /// let buffer: Buffer<_, 0> = [1u8, 2, 3, 4].into();
    ///
    /// assert_eq!(buffer.len(), 4);
    /// ```
    fn len(&self) -> usize;

    /// Returns `true` when the [length](Data::len) (number of elements) is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Buffer, Data};
    ///
    /// let empty_buffer = Buffer::<u8, 0>::empty();
    ///
    /// assert!(empty_buffer.is_empty());
    /// assert_eq!(empty_buffer.len(), 0);
    /// ```
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of null elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{ALIGNMENT, Buffer, Data, Nullable};
    ///
    /// let nullable: Nullable<Buffer<u32, ALIGNMENT>> =
    ///     [Some(1u32), None, Some(3), Some(4)].into();
    ///
    /// assert_eq!(nullable.null_count(), 1);
    /// ```
    fn null_count(&self) -> usize;

    /// Returns the number of non-null elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{ALIGNMENT, Buffer, Data, Nullable};
    ///
    /// let nullable: Nullable<Buffer<u32, ALIGNMENT>> =
    ///     [Some(1u32), None, Some(3), Some(4)].into();
    ///
    /// assert_eq!(nullable.valid_count(), 3);
    /// ```
    fn valid_count(&self) -> usize {
        self.len() - self.null_count()
    }
}

/// Types storing offset values.
///
/// This trait is sealed to prevent downstream implementations.
pub trait OffsetType: Primitive + crate::sealed::SealedOffsetType {}

impl OffsetType for i32 {}
impl OffsetType for i64 {}

// Sealed traits.
mod sealed {
    pub trait SealedPrimitive {}
    impl<T> SealedPrimitive for T where T: crate::Primitive {}

    pub trait SealedData {}
    impl<T> SealedData for T where T: crate::Data {}

    pub trait SealedOffsetType {}
    impl<T> SealedOffsetType for T where T: crate::OffsetType {}
}
