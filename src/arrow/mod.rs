//! Interop with the [`arrow-rs`] crate.
//!
//! [`arrow-rs`]: https://crates.io/crates/arrow

mod array;
pub use array::{StructArrayTypeFields, UnionArrayTypeFields};

mod bitmap;

pub mod buffer;

/// Extension trait of [`Array`] for [`arrow-rs`] interop.
///
/// [`arrow-rs`]: https://crates.io/crates/arrow
pub trait Array: crate::array::Array + Sized {
    /// The corresponding arrow array
    type Array: arrow_array::Array;

    /// Returns the field of this array.
    fn as_field(name: &str) -> arrow_schema::Field;

    /// Returns the data type of this array.
    fn data_type() -> arrow_schema::DataType;
}

/// Extension trait of [`LogicalArrayType`] for [`arrow-rs`] interop.
///
/// [`arrow-rs`]: https://crates.io/crates/arrow
pub trait LogicalArrayType<T>: crate::logical::LogicalArrayType<T>
where
    Self: crate::array::ArrayType<Self>,
    Option<Self>: crate::array::ArrayType<Self>,
{
    /// Arrow extension type. Use `()` if there is none.
    type ExtensionType: arrow_schema::extension::ExtensionType;

    /// Returns the `[arrow_schema::ExtensionType`] of this logical type, if
    /// there is one.
    #[must_use]
    fn extension_type() -> Option<Self::ExtensionType> {
        None
    }
}

/// An extension type implementation for types without extension types.
#[derive(Debug, Clone, Copy)]
pub struct NoExtensionType;

impl arrow_schema::extension::ExtensionType for NoExtensionType {
    const NAME: &'static str = "";

    type Metadata = ();

    fn metadata(&self) -> &Self::Metadata {
        panic!("should not be used")
    }

    fn serialize_metadata(&self) -> Option<String> {
        panic!("should not be used")
    }

    fn deserialize_metadata(
        _metadata: Option<&str>,
    ) -> Result<Self::Metadata, arrow_schema::ArrowError> {
        panic!("should not be used")
    }

    fn supports_data_type(
        &self,
        _data_type: &arrow_schema::DataType,
    ) -> Result<(), arrow_schema::ArrowError> {
        panic!("should not be used")
    }

    fn try_new(
        _data_type: &arrow_schema::DataType,
        _metadata: Self::Metadata,
    ) -> Result<Self, arrow_schema::ArrowError> {
        panic!("should not be used")
    }
}

/// Extension trait for [`Offset`] for [`arrow-rs`] interop.
///
/// [`arrow-rs`]: https://crates.io/crates/arrow
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
