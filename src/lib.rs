//! # narrow
//!
//! An implementation of [Apache Arrow](https://arrow.apache.org).

#![feature(generic_associated_types)]
#![cfg_attr(feature = "extend_one", feature(extend_one))]
#![cfg_attr(feature = "simd", feature(portable_simd))]

mod primitive;
pub use primitive::Primitive;

mod length;
pub use length::Length;

mod null;
pub use null::Null;

mod buffers;
pub use buffers::*;

// pub mod array;
pub mod bitmap;
pub mod buffer;
pub mod nullable;
pub mod offset;

// sorted<T> - just a wrapper
// impl min/max etc
// impl reverse for sorted

// nullinfo<T> - wrapper + some data i.e. counts
// impl null for nullinfo (e.g. short-circuit counts)

// mod validity;
// pub use validity::*;

// mod array;
// pub use array::*;

// mod compute;
// pub use compute::*;

// // Export derive macro(s).
// pub use narrow_derive::*;

// // Allow writing derive macro tests in this crate.
// extern crate self as narrow;
