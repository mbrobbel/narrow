//! Interop with the [`arrow-rs`] crate.
//!
//! [`arrow-rs`]: https://crates.io/crates/arrow

mod array;
pub use array::{StructArrayTypeFields, UnionArrayTypeFields};

mod bitmap;

pub mod buffer;

/// Extension trait of [`Array`] for [`arrow-rs`] interop.
pub trait Array: crate::array::Array + Sized {
    /// The corresponding arrow array
    type Array: arrow_array::Array;

    /// Returns the field of this array.
    fn as_field(name: &str) -> arrow_schema::Field;
}
