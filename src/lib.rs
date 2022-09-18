//! # narrow
//!
//! An implementation of [Apache Arrow](https://arrow.apache.org).

#![deny(warnings)]
#![cfg_attr(feature = "extend_one", feature(extend_one))]
#![cfg_attr(feature = "portable_simd", feature(portable_simd))]
#![cfg_attr(feature = "trusted_len", feature(trusted_len))]

mod primitive;
pub use primitive::Primitive;

mod length;
pub use length::Length;

mod null;
pub use null::Null;

mod buffers;
pub use buffers::*;

pub mod bitmap;
pub mod buffer;
pub mod nullable;
pub mod offset;
pub mod validity;
