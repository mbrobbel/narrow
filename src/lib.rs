//! # narrow
//!
//! An implementation of [Apache Arrow](https://arrow.apache.org).

mod primitive;
pub use primitive::*;

mod length;
pub use length::*;

mod buffer;
pub use buffer::*;

mod bitmap;
pub use bitmap::*;

mod null;
pub use null::*;

mod nullable;
pub use nullable::*;

mod validity;
pub use validity::*;

mod offset;
pub use offset::*;

mod array;
pub use array::*;

// // Export derive macro(s).
// pub use narrow_derive::*;

// // Allow writing derive macro tests in this crate.
// extern crate self as narrow;
