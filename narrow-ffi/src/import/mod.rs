//! Borrow Arrow C Data Interface arrays as Narrow arrays.

use core::fmt;

use narrow::{array::Array, buffer::SliceBuffer, layout::ArrayItem};

use crate::{ArrowArray, ArrowSchema};

/// Borrowed import support for an Arrow C Data array.
pub trait Import<'array>: Sized {
    /// Imports an [`ArrowArray`] and [`ArrowSchema`] without copying buffers.
    ///
    /// # Errors
    ///
    /// Returns an [`ImportError`] when the structures do not describe the
    /// expected Narrow array representation.
    ///
    /// # Safety
    ///
    /// The Arrow structures and every referenced buffer must satisfy the Arrow
    /// C Data Interface contract. Referenced buffers must remain immutable for
    /// `'array` and contain enough properly aligned elements for the declared
    /// array length.
    unsafe fn import(array: &'array ArrowArray, schema: &ArrowSchema) -> Result<Self, ImportError>;
}

/// A memory layout that can borrow an Arrow C Data array.
trait ImportLayout<'array>: Sized {
    /// Imports the memory layout described by an Arrow array and schema.
    ///
    /// # Safety
    ///
    /// The caller must uphold the requirements of [`Import::import`].
    unsafe fn import_layout(
        array: &'array ArrowArray,
        schema: &ArrowSchema,
    ) -> Result<Self, ImportError>;
}

impl<'array, T> Import<'array> for Array<T, SliceBuffer<'array>>
where
    T: ArrayItem,
    T::Memory<SliceBuffer<'array>>: ImportLayout<'array>,
{
    unsafe fn import(array: &'array ArrowArray, schema: &ArrowSchema) -> Result<Self, ImportError> {
        // SAFETY: The caller upholds the requirements of `Import::import`.
        let memory = unsafe {
            <T::Memory<SliceBuffer<'array>> as ImportLayout>::import_layout(array, schema)
        }?;
        Ok(Self::from_buffer(memory))
    }
}

/// Error returned when Arrow C Data cannot be borrowed as a Narrow array.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImportError {
    /// The Arrow array was already released.
    ReleasedArray,
    /// The Arrow schema was already released.
    ReleasedSchema,
    /// The Arrow schema does not contain a format string.
    MissingFormat,
    /// The Arrow format does not match the requested Narrow array type.
    UnexpectedFormat,
    /// The Arrow schema has unsupported flags.
    UnexpectedFlags {
        /// Schema flags supplied by the producer.
        flags: i64,
    },
    /// The Arrow array length is negative or does not fit in [`usize`].
    InvalidLength {
        /// Invalid array length supplied by the producer.
        length: i64,
    },
    /// The Arrow array has an unsupported offset.
    NonZeroOffset {
        /// Unsupported offset supplied by the producer.
        offset: i64,
    },
    /// A non-nullable array reports a non-zero null count.
    UnexpectedNullCount {
        /// Null count supplied by the producer.
        null_count: i64,
    },
    /// The Arrow array has an unexpected number of buffers.
    UnexpectedBufferCount {
        /// Buffer count supplied by the producer.
        count: i64,
    },
    /// The Arrow array or schema has an unexpected number of children.
    UnexpectedChildCount {
        /// Array child count supplied by the producer.
        array: i64,
        /// Schema child count supplied by the producer.
        schema: i64,
    },
    /// The Arrow array or schema contains an unexpected dictionary.
    UnexpectedDictionary,
    /// The Arrow array does not contain its buffer pointer array.
    MissingBufferPointers,
    /// A non-empty array does not contain a value buffer.
    MissingValuesBuffer,
    /// The primitive value buffer is not aligned for its element type.
    MisalignedValuesBuffer {
        /// Required byte alignment of the requested element type.
        alignment: usize,
    },
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ReleasedArray => write!(f, "Arrow array is already released"),
            Self::ReleasedSchema => write!(f, "Arrow schema is already released"),
            Self::MissingFormat => write!(f, "Arrow schema format is missing"),
            Self::UnexpectedFormat => write!(f, "Arrow schema format does not match"),
            Self::UnexpectedFlags { flags } => {
                write!(f, "Arrow schema flags ({flags}) are not supported")
            }
            Self::InvalidLength { length } => {
                write!(f, "Arrow array length ({length}) is invalid")
            }
            Self::NonZeroOffset { offset } => {
                write!(f, "Arrow array offset ({offset}) is not supported")
            }
            Self::UnexpectedNullCount { null_count } => {
                write!(f, "non-nullable Arrow array has null count {null_count}")
            }
            Self::UnexpectedBufferCount { count } => {
                write!(f, "Arrow array buffer count ({count}) does not match")
            }
            Self::UnexpectedChildCount { array, schema } => write!(
                f,
                "Arrow child counts do not match: array {array}, schema {schema}"
            ),
            Self::UnexpectedDictionary => {
                write!(f, "Arrow dictionary is not supported for this array")
            }
            Self::MissingBufferPointers => write!(f, "Arrow buffer pointers are missing"),
            Self::MissingValuesBuffer => write!(f, "Arrow value buffer is missing"),
            Self::MisalignedValuesBuffer { alignment } => write!(
                f,
                "Arrow value buffer does not have the required alignment ({alignment})"
            ),
        }
    }
}

impl core::error::Error for ImportError {}

/// Borrowed import support for Boolean arrays.
mod boolean;
/// Borrowed import support for fixed-size primitive arrays.
mod fixed_size_primitive;
