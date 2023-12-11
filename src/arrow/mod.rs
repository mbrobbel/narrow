//! Interop with the [`arrow-rs`] crate.
//!
//! [`arrow-rs`]: https://crates.io/crates/arrow

mod array;
pub use array::StructArrayTypeFields;

mod buffer;
pub use buffer::*;

use crate::array::Array;
use arrow_schema::Field;

/// Extension trait of [`Array`] for [`arrow-rs`] interop.
pub trait ArrowArray: Array + Sized {
    /// The corresponding arrow array
    type Array: arrow_array::Array;

    /// Returns the field of this array.
    fn as_field(name: &str) -> Field;
}
