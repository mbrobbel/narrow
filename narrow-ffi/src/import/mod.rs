//! Borrow Arrow C Data Interface arrays as Narrow arrays.

use core::{ffi::CStr, fmt};

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
    /// Every non-null pointer read by the importer must be valid for the
    /// required reads. Referenced buffers must remain immutable for `'array`
    /// and contain enough properly aligned elements for the declared array
    /// length. Scalar metadata is validated by the importer.
    unsafe fn import(array: &'array ArrowArray, schema: &ArrowSchema) -> Result<Self, ImportError>;
}

/// A memory layout that can borrow an Arrow C Data array.
trait ImportLayout<'array>: Sized {
    /// Expected Arrow schema flags.
    const FLAGS: i64 = 0;
    /// Expected number of Arrow array buffers.
    const N_BUFFERS: i64;
    /// Expected number of Arrow array and schema children.
    const N_CHILDREN: i64;

    /// Returns whether an Arrow format matches this memory layout.
    fn matches_format(format: &CStr) -> bool;

    /// Constructs the memory layout after its common fields are validated.
    ///
    /// # Safety
    ///
    /// The caller must uphold the requirements of [`Import::import`].
    unsafe fn import_validated(
        array: &'array ArrowArray,
        schema: &ArrowSchema,
        length: usize,
    ) -> Result<Self, ImportError>;

    /// Validates and imports the memory layout described by an Arrow array and
    /// schema.
    ///
    /// # Safety
    ///
    /// The caller must uphold the requirements of [`Import::import`].
    unsafe fn import_layout(
        array: &'array ArrowArray,
        schema: &ArrowSchema,
    ) -> Result<Self, ImportError> {
        if array.is_released() {
            return Err(ImportError::ReleasedArray);
        }
        if schema.is_released() {
            return Err(ImportError::ReleasedSchema);
        }
        if schema.format.is_null() {
            return Err(ImportError::MissingFormat);
        }
        // SAFETY: The caller guarantees a valid null-terminated schema format.
        if !Self::matches_format(unsafe { CStr::from_ptr(schema.format) }) {
            return Err(ImportError::UnexpectedFormat);
        }
        if schema.flags != Self::FLAGS {
            return Err(ImportError::UnexpectedFlags {
                flags: schema.flags,
            });
        }

        let length = usize::try_from(array.length).map_err(|_| ImportError::InvalidLength {
            length: array.length,
        })?;
        if array.offset != 0 {
            return Err(ImportError::NonZeroOffset {
                offset: array.offset,
            });
        }
        if array.null_count != 0 {
            return Err(ImportError::UnexpectedNullCount {
                null_count: array.null_count,
            });
        }
        if array.n_buffers != Self::N_BUFFERS {
            return Err(ImportError::UnexpectedBufferCount {
                count: array.n_buffers,
            });
        }
        if array.n_children != Self::N_CHILDREN || schema.n_children != Self::N_CHILDREN {
            return Err(ImportError::UnexpectedChildCount {
                array: array.n_children,
                schema: schema.n_children,
            });
        }
        if !array.dictionary.is_null() || !schema.dictionary.is_null() {
            return Err(ImportError::UnexpectedDictionary);
        }
        if Self::N_BUFFERS != 0 && array.buffers.is_null() {
            return Err(ImportError::MissingBufferPointers);
        }
        if Self::N_CHILDREN != 0 && array.children.is_null() {
            return Err(ImportError::MissingArrayChildren);
        }
        if Self::N_CHILDREN != 0 && schema.children.is_null() {
            return Err(ImportError::MissingSchemaChildren);
        }

        // SAFETY: Common fields have been validated and the caller upholds the
        // Arrow C Data pointer and buffer requirements.
        unsafe { Self::import_validated(array, schema, length) }
    }
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
    /// The Arrow array does not contain its child pointer array.
    MissingArrayChildren,
    /// The Arrow schema does not contain its child pointer array.
    MissingSchemaChildren,
    /// The fixed-size-list width is not supported.
    UnsupportedFixedSizeListSize {
        /// Unsupported fixed-size-list width.
        size: usize,
    },
    /// A fixed-size-list child length does not match its parent.
    FixedSizeListLengthMismatch {
        /// Number of items in the parent array.
        length: usize,
        /// Number of items in the child array.
        child_length: usize,
        /// Number of child items per parent item.
        size: usize,
    },
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
                "Arrow child counts do not match the imported layout: array {array}, schema {schema}"
            ),
            Self::UnexpectedDictionary => {
                write!(f, "Arrow dictionary is not supported for this array")
            }
            Self::MissingBufferPointers => write!(f, "Arrow buffer pointers are missing"),
            Self::MissingArrayChildren => write!(f, "Arrow array child pointers are missing"),
            Self::MissingSchemaChildren => write!(f, "Arrow schema child pointers are missing"),
            Self::UnsupportedFixedSizeListSize { size } => {
                write!(f, "fixed-size-list width ({size}) is not supported")
            }
            Self::FixedSizeListLengthMismatch {
                length,
                child_length,
                size,
            } => write!(
                f,
                "fixed-size-list length ({length}) with width ({size}) does not match child length ({child_length})"
            ),
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
/// Borrowed import support for fixed-size-list arrays.
mod fixed_size_list;
/// Borrowed import support for fixed-size primitive arrays.
mod fixed_size_primitive;
