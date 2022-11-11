//! # narrow
//!
//! An experimental (work-in-progress) implementation of [Apache Arrow](https://arrow.apache.org).

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/mbrobbel/narrow/main/narrow.svg",
    html_favicon_url = "https://raw.githubusercontent.com/mbrobbel/narrow/main/narrow.svg"
)]
#![deny(warnings)]

mod primitive;
pub use primitive::Primitive;

mod length;
pub use length::Length;

pub mod bitmap;
pub mod buffer;

pub(crate) mod nullable;
pub(crate) mod offset;
pub(crate) mod validity;

pub mod array;

// Re-export `narrow_derive` macros when the `derive` feature is enabled.
#[cfg(feature = "derive")]
pub use narrow_derive::Array;
