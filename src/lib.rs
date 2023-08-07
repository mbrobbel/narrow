//! # narrow
//!
//! An experimental (work-in-progress) implementation of [Apache Arrow](https://arrow.apache.org).

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/mbrobbel/narrow/main/narrow.svg",
    html_favicon_url = "https://raw.githubusercontent.com/mbrobbel/narrow/main/narrow.svg"
)]
#![deny(warnings)]
// #![deny(
//     missing_copy_implementations,
//     missing_debug_implementations,
//     missing_docs
// )]

mod fixed_size;
pub use self::fixed_size::FixedSize;

mod length;
pub use self::length::Length;

pub mod buffer;

pub mod bitmap;

pub(crate) mod nullable;
pub(crate) mod offset;
pub(crate) mod validity;

pub mod array;

#[cfg(any(feature = "arrow-array", feature = "arrow-buffer"))]
pub mod arrow;

// Re-export `narrow_derive` macros when the `derive` feature is enabled.
#[cfg(feature = "derive")]
pub use narrow_derive::ArrayType;
