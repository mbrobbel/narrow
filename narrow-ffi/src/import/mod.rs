//! Borrow Arrow C Data Interface arrays as Narrow arrays.

use core::{ffi::CStr, fmt, marker::PhantomData, slice};

use narrow::{
    array::Array,
    bitmap::{Bitmap, ValidityBitmap},
    buffer::SliceBuffer,
    collection::Collection,
    layout::ArrayItem,
    nullability::{NonNullable, Nullability, Nullable},
    offset::OffsetsError,
    validity::Validity,
};

use crate::{ARROW_FLAG_NULLABLE, ArrowArray, ArrowSchema};

/// Borrowed import support for an Arrow C Data array.
pub trait Import<'array>: Sized {
    /// Validates an [`ArrowSchema`] for this array type.
    ///
    /// # Errors
    ///
    /// Returns an [`ImportError`] when the schema does not describe the
    /// expected array representation.
    ///
    /// # Safety
    ///
    /// Every non-null pointer read from `schema` must be valid for the required
    /// reads, including null-terminated format strings and child schemas.
    unsafe fn validate_schema(schema: &ArrowSchema) -> Result<(), ImportError>;

    /// Imports an [`ArrowArray`] after its schema is validated.
    ///
    /// # Errors
    ///
    /// Returns an [`ImportError`] when the array does not match the expected
    /// representation.
    ///
    /// # Safety
    ///
    /// Every non-null pointer read from `array` must be valid for the required
    /// reads. Referenced buffers must remain immutable for `'array` and contain
    /// enough properly aligned elements for the declared array length. The
    /// array must conform to the schema validated for this array type.
    unsafe fn import_array(array: &'array ArrowArray) -> Result<Self, ImportError>;

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
    unsafe fn import(array: &'array ArrowArray, schema: &ArrowSchema) -> Result<Self, ImportError> {
        // SAFETY: The caller upholds the schema requirements of this method.
        unsafe { Self::validate_schema(schema) }?;
        // SAFETY: The caller upholds the array requirements of this method.
        unsafe { Self::import_array(array) }
    }
}

/// A reusable importer for a validated Arrow schema.
///
/// The schema is not retained and may be released after construction.
///
/// ```
/// use narrow::{array::Array, buffer::SliceBuffer, collection::Collection};
/// use narrow_ffi::{Export, Importer};
///
/// let source = [1_i32, 2, 3].into_iter().collect::<Array<i32>>();
/// let (array, schema) = source.export().unwrap();
///
/// // SAFETY: The exported schema and its format string are valid.
/// let importer = unsafe { Importer::<i32>::try_new(&schema) }.unwrap();
/// drop(schema);
///
/// // SAFETY: The exported array owns a valid primitive value buffer.
/// let imported: Array<i32, SliceBuffer<'_>> =
///     unsafe { importer.import(&array) }.unwrap();
/// assert_eq!(imported.owned(1), Some(2));
/// ```
pub struct Importer<T> {
    /// Imported array item type.
    item: PhantomData<fn() -> T>,
}

impl<T> Clone for Importer<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Importer<T> {}

impl<T> fmt::Debug for Importer<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Importer").finish()
    }
}

impl<T> Importer<T>
where
    T: ArrayItem,
    for<'array> Array<T, SliceBuffer<'array>>: Import<'array>,
{
    /// Validates an Arrow schema for arrays of `T`.
    ///
    /// # Errors
    ///
    /// Returns an [`ImportError`] when the schema does not describe the
    /// expected array representation.
    ///
    /// # Safety
    ///
    /// Every non-null pointer read from `schema` must be valid for the required
    /// reads, including null-terminated format strings and child schemas.
    pub unsafe fn try_new(schema: &ArrowSchema) -> Result<Self, ImportError> {
        // SAFETY: The caller upholds the schema requirements of this method.
        unsafe { <Array<T, SliceBuffer<'static>> as Import<'static>>::validate_schema(schema) }?;
        Ok(Self { item: PhantomData })
    }

    /// Borrows an Arrow array without revalidating its schema.
    ///
    /// # Errors
    ///
    /// Returns an [`ImportError`] when the array does not match the validated
    /// schema.
    ///
    /// # Safety
    ///
    /// Every non-null pointer read from `array` must be valid for the required
    /// reads. Referenced buffers must remain immutable for `'array` and contain
    /// enough properly aligned elements for the declared array length. The
    /// array must conform to the schema used to construct this importer.
    pub unsafe fn import<'array>(
        &self,
        array: &'array ArrowArray,
    ) -> Result<Array<T, SliceBuffer<'array>>, ImportError> {
        // SAFETY: The caller upholds the array requirements of this method.
        unsafe { <Array<T, SliceBuffer<'array>> as Import<'array>>::import_array(array) }
    }
}

/// Arrow import behavior for a [`Nullability`] type constructor.
trait ImportNullability<'array>: Nullability {
    /// Arrow schema nullable flag for this nullability.
    const FLAGS: i64;

    /// Wraps an imported collection with this nullability.
    ///
    /// # Safety
    ///
    /// Common fields must be validated, and the caller must uphold the
    /// requirements of [`Import::import`] for the validity buffer.
    unsafe fn wrap<T>(
        array: &'array ArrowArray,
        collection: T,
    ) -> Result<Self::Collection<T, SliceBuffer<'array>>, ImportError>
    where
        T: Collection;
}

impl<'array> ImportNullability<'array> for NonNullable {
    const FLAGS: i64 = 0;

    unsafe fn wrap<T>(
        _array: &'array ArrowArray,
        collection: T,
    ) -> Result<Self::Collection<T, SliceBuffer<'array>>, ImportError>
    where
        T: Collection,
    {
        Ok(collection)
    }
}

impl<'array> ImportNullability<'array> for Nullable {
    const FLAGS: i64 = ARROW_FLAG_NULLABLE;

    unsafe fn wrap<T>(
        array: &'array ArrowArray,
        collection: T,
    ) -> Result<Self::Collection<T, SliceBuffer<'array>>, ImportError>
    where
        T: Collection,
    {
        // Common validation guarantees a non-null buffer pointer array.
        let buffers = usize::try_from(array.n_buffers).expect("buffer count must fit in usize");
        assert!(buffers != 0, "validity requires a buffer");

        // SAFETY: Common validation and the caller guarantee a valid buffer
        // pointer array with `buffers` entries.
        let buffer_pointers = unsafe { slice::from_raw_parts(array.buffers, buffers) };
        let validity_pointer = buffer_pointers[0].cast::<u8>();
        let length = collection.len();
        let validity = if validity_pointer.is_null() {
            if array.null_count == 0 || length == 0 {
                Validity::from_collection(collection)
            } else {
                return Err(ImportError::MissingValidityBuffer);
            }
        } else {
            let byte_length = length.div_ceil(8);
            // SAFETY: The caller guarantees the validity buffer contains
            // `byte_length` bytes that remain immutable for `'array`.
            let values = unsafe { slice::from_raw_parts(validity_pointer, byte_length) };
            let bitmap = Bitmap::try_from_parts(values, length, 0)
                .expect("imported validity buffer contains the declared number of bits");
            Validity::try_from_parts(collection, bitmap).expect("validity lengths match")
        };

        if array.null_count >= 0 {
            let actual = validity.null_count();
            if usize::try_from(array.null_count) != Ok(actual) {
                return Err(ImportError::NullCountMismatch {
                    declared: array.null_count,
                    actual,
                });
            }
        }

        Ok(validity)
    }
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
        length: usize,
    ) -> Result<Self, ImportError>;

    /// Validates child schemas after the common parent fields are validated.
    ///
    /// # Safety
    ///
    /// Every child schema pointer read from `schema` must be valid for the
    /// required reads.
    unsafe fn validate_child_schemas(_schema: &ArrowSchema) -> Result<(), ImportError> {
        Ok(())
    }

    /// Validates an Arrow schema for this memory layout.
    ///
    /// # Safety
    ///
    /// Every non-null pointer read from `schema` must be valid for the required
    /// reads, including null-terminated format strings and child schemas.
    unsafe fn validate_schema(schema: &ArrowSchema) -> Result<(), ImportError> {
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
        if schema.n_children != Self::CHILDREN {
            return Err(ImportError::UnexpectedSchemaChildCount {
                count: schema.n_children,
            });
        }
        if !schema.dictionary.is_null() {
            return Err(ImportError::UnexpectedDictionary);
        }
        if Self::CHILDREN != 0 && schema.children.is_null() {
            return Err(ImportError::MissingSchemaChildren);
        }

        // SAFETY: Common schema fields have been validated and the caller
        // upholds the Arrow C Data pointer requirements.
        unsafe { Self::validate_child_schemas(schema) }
    }

    /// Validates and imports an Arrow array for this memory layout.
    ///
    /// # Safety
    ///
    /// The caller must uphold the array and buffer requirements of
    /// [`Importer::import`].
    unsafe fn import_array(array: &'array ArrowArray) -> Result<Self, ImportError> {
        if array.is_released() {
            return Err(ImportError::ReleasedArray);
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
        if array.n_children != Self::CHILDREN {
            return Err(ImportError::UnexpectedArrayChildCount {
                count: array.n_children,
            });
        }
        if !array.dictionary.is_null() {
            return Err(ImportError::UnexpectedDictionary);
        }
        if Self::BUFFERS != 0 && array.buffers.is_null() {
            return Err(ImportError::MissingBufferPointers);
        }
        if Self::CHILDREN != 0 && array.children.is_null() {
            return Err(ImportError::MissingArrayChildren);
        }

        // SAFETY: Common fields have been validated and the caller upholds the
        // Arrow C Data pointer and buffer requirements.
        unsafe { Self::import_validated(array, length) }
    }

    /// Validates a child schema at `index`.
    ///
    /// # Safety
    ///
    /// Common parent fields must be validated, and every child schema pointer
    /// read from `schema` must be valid for the required reads.
    unsafe fn validate_child_schema<Child>(
        schema: &ArrowSchema,
        index: usize,
    ) -> Result<(), ImportError>
    where
        Child: ImportLayout<'array>,
    {
        let children = usize::try_from(Self::CHILDREN).expect("child count must fit in usize");
        assert!(index < children, "child index is out of bounds");

        // SAFETY: Common validation and the caller guarantee a valid child
        // pointer array with `children` entries.
        let schema_children = unsafe { slice::from_raw_parts(schema.children, children) };
        let child_schema_pointer = schema_children[index];
        if child_schema_pointer.is_null() {
            return Err(ImportError::MissingSchemaChildren);
        }
        // SAFETY: The caller guarantees that the child schema is valid for the
        // duration of this validation.
        let child_schema = unsafe { &*child_schema_pointer };

        // SAFETY: The child schema is covered by the caller's Arrow C Data
        // guarantees.
        unsafe { Child::validate_schema(child_schema) }
    }

    /// Imports a child memory layout at `index`.
    ///
    /// # Safety
    ///
    /// Common parent fields must be validated, and the caller must uphold the
    /// requirements of [`Import::import`] for the child structures.
    unsafe fn import_child<Child>(
        array: &'array ArrowArray,
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

        // SAFETY: The child array is covered by the caller's Arrow C Data
        // guarantees and retained by its parent.
        unsafe { Child::import_array(child_array) }
    }
}

impl<'array, T> Import<'array> for Array<T, SliceBuffer<'array>>
where
    T: ArrayItem,
    T::Memory<SliceBuffer<'array>>: ImportLayout<'array>,
{
    unsafe fn validate_schema(schema: &ArrowSchema) -> Result<(), ImportError> {
        // SAFETY: The caller upholds the requirements of this method.
        unsafe { <T::Memory<SliceBuffer<'array>> as ImportLayout>::validate_schema(schema) }?;
        Ok(())
    }

    unsafe fn import_array(array: &'array ArrowArray) -> Result<Self, ImportError> {
        // SAFETY: The caller upholds the requirements of this method.
        let memory =
            unsafe { <T::Memory<SliceBuffer<'array>> as ImportLayout>::import_array(array) }?;
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
    /// A nullable non-empty array does not contain a required validity buffer.
    MissingValidityBuffer,
    /// A nullable array's null count does not match its validity bitmap.
    NullCountMismatch {
        /// Null count supplied by the producer.
        declared: i64,
        /// Number of nulls found in the validity bitmap.
        actual: usize,
    },
    /// The Arrow array has an unexpected number of buffers.
    UnexpectedBufferCount {
        /// Buffer count supplied by the producer.
        count: i64,
    },
    /// The Arrow array has an unexpected number of children.
    UnexpectedArrayChildCount {
        /// Array child count supplied by the producer.
        count: i64,
    },
    /// The Arrow schema has an unexpected number of children.
    UnexpectedSchemaChildCount {
        /// Schema child count supplied by the producer.
        count: i64,
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
            Self::MissingValidityBuffer => {
                write!(f, "nullable Arrow array validity buffer is missing")
            }
            Self::NullCountMismatch { declared, actual } => write!(
                f,
                "Arrow array null count ({declared}) does not match validity bitmap ({actual})"
            ),
            Self::UnexpectedBufferCount { count } => {
                write!(f, "Arrow array buffer count ({count}) does not match")
            }
            Self::UnexpectedArrayChildCount { count } => {
                write!(f, "Arrow array child count ({count}) does not match")
            }
            Self::UnexpectedSchemaChildCount { count } => {
                write!(f, "Arrow schema child count ({count}) does not match")
            }
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

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::{vec, vec::Vec};

    use narrow::{array::Array, buffer::SliceBuffer, collection::Collection};

    use crate::{Export, ImportError, Importer};

    #[test]
    fn imports_multiple_arrays_after_releasing_schema() {
        let first_source = [vec![1_i32, 2], vec![3]]
            .into_iter()
            .collect::<Array<Vec<i32>>>();
        let second_source = [vec![4_i32], vec![5, 6]]
            .into_iter()
            .collect::<Array<Vec<i32>>>();
        let (first_array, schema) = first_source.export().expect("export first array");
        let (second_array, second_schema) = second_source.export().expect("export second array");

        // SAFETY: The exported schema and its child schema are valid.
        let importer = unsafe { Importer::<Vec<i32>>::try_new(&schema) }.expect("validate schema");
        drop(schema);
        drop(second_schema);

        // SAFETY: Both exported arrays conform to the validated schema and
        // retain their buffers for the imported borrows.
        let first: Array<Vec<i32>, SliceBuffer<'_>> =
            unsafe { importer.import(&first_array) }.expect("import first array");
        // SAFETY: Both exported arrays conform to the validated schema and
        // retain their buffers for the imported borrows.
        let second: Array<Vec<i32>, SliceBuffer<'_>> =
            unsafe { importer.import(&second_array) }.expect("import second array");

        assert_eq!(first.owned(0), Some(vec![1, 2]));
        assert_eq!(first.owned(1), Some(vec![3]));
        assert_eq!(second.owned(0), Some(vec![4]));
        assert_eq!(second.owned(1), Some(vec![5, 6]));
    }

    #[test]
    fn rejects_schema_before_importing_an_array() {
        let source = [true, false].into_iter().collect::<Array<bool>>();
        let (_array, schema) = source.export().expect("export array");

        // SAFETY: The exported Boolean schema is valid but does not describe
        // the requested primitive type.
        let error = unsafe { Importer::<i32>::try_new(&schema) }.expect_err("mismatched schema");

        assert_eq!(error, ImportError::UnexpectedFormat);
    }

    #[test]
    fn validates_nested_child_schemas() {
        let source = [vec![1_i32, 2]].into_iter().collect::<Array<Vec<i32>>>();
        let (_array, schema) = source.export().expect("export array");
        // SAFETY: The exported schema owns a readable one-entry child pointer
        // array containing a live child schema.
        let child_pointer = unsafe { *schema.children };
        // SAFETY: The child pointer refers to a live, writable child schema.
        let child = unsafe { &mut *child_pointer };
        child.format = c"g".as_ptr();

        // SAFETY: The schema and child pointers remain valid; the child format
        // mismatch is validated by the importer.
        let error =
            unsafe { Importer::<Vec<i32>>::try_new(&schema) }.expect_err("mismatched child schema");

        assert_eq!(error, ImportError::UnexpectedFormat);
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
