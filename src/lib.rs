//! # narrow
//!
//! An implementation of [Apache Arrow](https://arrow.apache.org).

#![feature(generic_associated_types)]
#![feature(new_uninit)]
#![cfg(feature = "trusted_len")]
#![feature(trusted_len)]

mod primitive;
pub use primitive::*;

mod length;
pub use length::*;

mod buffer;
pub use buffer::*;

mod fmt;
pub(crate) use fmt::*;

mod iter;
pub use iter::*;

mod bitmap;
pub use bitmap::*;

mod buffers;
pub use buffers::*;

mod null;
pub use null::*;

// mod nullable;
// pub use nullable::*;

// mod validity;
// pub use validity::*;

// mod offset;
// pub use offset::*;

mod array;
pub use array::*;

// mod compute;
// pub use compute::*;

// // Export derive macro(s).
// pub use narrow_derive::*;

// // Allow writing derive macro tests in this crate.
// extern crate self as narrow;
