//! # narrow
//!
//! A Rust implementation of [Apache Arrow](https://arrow.apache.org).

mod primitive;
pub use primitive::*;

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

mod array;
pub use array::*;

#[cfg(feature = "derive")]
// Export derive macro(s).
pub use narrow_derive::*;

// Allow writing derive macro tests in this crate.
extern crate self as narrow;
