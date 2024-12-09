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

    /// Returns the data type of this array.
    fn data_type() -> arrow_schema::DataType;
}

/// Extension trait for [`Offset`] for [`arrow-rs`] interop.
pub trait Offset: crate::offset::Offset {
    /// This constant is true when this offset maps to the large variant of a
    /// datatype.
    const LARGE: bool;
}

impl Offset for i32 {
    const LARGE: bool = false;
}

impl Offset for i64 {
    const LARGE: bool = true;
}
