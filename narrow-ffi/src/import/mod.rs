//! Borrow Arrow C Data Interface arrays as Narrow arrays.

use core::{ffi::CStr, fmt, slice};

use narrow::{array::Array, buffer::SliceBuffer, layout::ArrayItem, offset::OffsetsError};

use crate::{ARROW_FLAG_NULLABLE, ArrowArray, ArrowSchema};

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
    /// Expected Arrow schema nullable flag.
    const FLAGS: i64 = 0;
    /// Expected number of Arrow array buffers.
    const BUFFERS: i64;
    /// Expected number of Arrow array and schema children.
    const CHILDREN: i64;

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
        if schema.flags & ARROW_FLAG_NULLABLE != Self::FLAGS {
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
        // Arrow permits -1 when the null count has not been computed. Known
        // counts must fit the array, and non-nullable layouts cannot contain
        // known null items.
        if array.null_count < -1
            || array.null_count > array.length
            || (Self::FLAGS == 0 && array.null_count > 0)
        {
            return Err(ImportError::UnexpectedNullCount {
                null_count: array.null_count,
            });
        }
        if array.n_buffers != Self::BUFFERS {
            return Err(ImportError::UnexpectedBufferCount {
                count: array.n_buffers,
            });
        }
        if array.n_children != Self::CHILDREN || schema.n_children != Self::CHILDREN {
            return Err(ImportError::UnexpectedChildCount {
                array: array.n_children,
                schema: schema.n_children,
            });
        }
        if !array.dictionary.is_null() || !schema.dictionary.is_null() {
            return Err(ImportError::UnexpectedDictionary);
        }
        if Self::BUFFERS != 0 && array.buffers.is_null() {
            return Err(ImportError::MissingBufferPointers);
        }
        if Self::CHILDREN != 0 && array.children.is_null() {
            return Err(ImportError::MissingArrayChildren);
        }
        if Self::CHILDREN != 0 && schema.children.is_null() {
            return Err(ImportError::MissingSchemaChildren);
        }

        // SAFETY: Common fields have been validated and the caller upholds the
        // Arrow C Data pointer and buffer requirements.
        unsafe { Self::import_validated(array, schema, length) }
    }

    /// Imports a child memory layout at `index`.
    ///
    /// # Safety
    ///
    /// Common parent fields must be validated, and the caller must uphold the
    /// requirements of [`Import::import`] for the child structures.
    unsafe fn import_child<Child>(
        array: &'array ArrowArray,
        schema: &ArrowSchema,
        index: usize,
    ) -> Result<Child, ImportError>
    where
        Child: ImportLayout<'array>,
    {
        let children = usize::try_from(Self::CHILDREN).expect("child count must fit in usize");
        assert!(index < children, "child index is out of bounds");

        // SAFETY: Common validation and the caller guarantee a valid child
        // pointer array with `children` entries.
        let array_children = unsafe { slice::from_raw_parts(array.children, children) };
        let child_array_pointer = array_children[index];
        if child_array_pointer.is_null() {
            return Err(ImportError::MissingArrayChildren);
        }
        // SAFETY: The caller guarantees that the child array is retained by
        // the parent for `'array`.
        let child_array: &'array ArrowArray = unsafe { &*child_array_pointer };

        // SAFETY: Common validation and the caller guarantee a valid child
        // pointer array with `children` entries.
        let schema_children = unsafe { slice::from_raw_parts(schema.children, children) };
        let child_schema_pointer = schema_children[index];
        if child_schema_pointer.is_null() {
            return Err(ImportError::MissingSchemaChildren);
        }
        // SAFETY: The caller guarantees that the child schema is valid while
        // the parent schema is borrowed.
        let child_schema = unsafe { &*child_schema_pointer };

        // SAFETY: The child structures are covered by the caller's Arrow C
        // Data guarantees and retained by their respective parents.
        unsafe { Child::import_layout(child_array, child_schema) }
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
    /// The Arrow schema nullable flag does not match the imported layout.
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
    /// The Arrow array null count does not match its length or imported layout.
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
    /// A non-empty offsets buffer is missing.
    MissingOffsetsBuffer,
    /// The offsets buffer is not aligned for its element type.
    MisalignedOffsetsBuffer {
        /// Required byte alignment of the offset type.
        alignment: usize,
    },
    /// An offsets buffer does not satisfy the Arrow offset invariants.
    InvalidOffsets {
        /// Offset invariant that was violated.
        error: OffsetsError,
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
                write!(
                    f,
                    "Arrow schema nullable flag in flags ({flags}) does not match"
                )
            }
            Self::InvalidLength { length } => {
                write!(f, "Arrow array length ({length}) is invalid")
            }
            Self::NonZeroOffset { offset } => {
                write!(f, "Arrow array offset ({offset}) is not supported")
            }
            Self::UnexpectedNullCount { null_count } => {
                write!(
                    f,
                    "Arrow array null count ({null_count}) does not match the imported layout"
                )
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
            Self::MissingOffsetsBuffer => write!(f, "Arrow offsets buffer is missing"),
            Self::MisalignedOffsetsBuffer { alignment } => write!(
                f,
                "Arrow offsets buffer does not have the required alignment ({alignment})"
            ),
            Self::InvalidOffsets { error } => write!(f, "invalid Arrow offsets: {error}"),
            Self::MissingValuesBuffer => write!(f, "Arrow value buffer is missing"),
            Self::MisalignedValuesBuffer { alignment } => write!(
                f,
                "Arrow value buffer does not have the required alignment ({alignment})"
            ),
        }
    }
}

impl core::error::Error for ImportError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match *self {
            Self::InvalidOffsets { ref error } => Some(error),
            _ => None,
        }
    }
}

/// Borrowed import support for Boolean arrays.
mod boolean;
/// Borrowed import support for fixed-size-list arrays.
mod fixed_size_list;
/// Borrowed import support for fixed-size primitive arrays.
mod fixed_size_primitive;
/// Borrowed import support for variable-size-list arrays.
mod variable_size_list;
