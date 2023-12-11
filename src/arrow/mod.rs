//! Interop with the [`arrow-rs`] crate.
//!
//! [`arrow-rs`]: https://crates.io/crates/arrow

mod array;
// mod bitmap;

mod buffer;
pub use buffer::*;

use crate::array::Array;
use arrow_array::ArrayRef;
use arrow_schema::Field;
use std::sync::Arc;

/// Extension trait of [`Array`] for [`arrow-rs`] interop.
pub trait ArrowArray: Array + Sized {
    /// The corresponding arrow array
    type Array: arrow_array::Array; // + From<Self> + 'static;

    /// Returns as array ref
    fn into_array_ref(self) -> ArrayRef
    where
        Self::Array: From<Self> + 'static,
    {
        Arc::<Self::Array>::new(self.into())
    }

    /// Returns the field of this array.
    fn as_field(&self, name: &str) -> Field;
}
