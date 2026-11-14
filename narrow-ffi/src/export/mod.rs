//! Export [`Array`] values through the Arrow C Data Interface.

extern crate alloc;

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{fmt, ptr};

use narrow::{array::Array, layout::ArrayItem, length::Length};

use crate::{ArrowArray, ArrowSchema, ArrowType};

use self::checked::{BufferPointer, Checker, Exported};
pub use self::trusted::{ExportBuffer, Trusted};

/// Export support for [`narrow::layout::boolean::Boolean`].
mod boolean;
/// Buffer pointers checked against the array that exports them.
mod checked;
/// Export support for [`narrow::layout::fixed_size_list::FixedSizeList`].
mod fixed_size_list;
/// Export support for [`narrow::layout::fixed_size_primitive::FixedSizePrimitive`].
mod fixed_size_primitive;
/// Storage that export may trust to skip checks that read every item.
mod trusted;
#[cfg(test)]
mod untrusted_buffer;
/// Export support for [`narrow::layout::variable_size_list::VariableSizeList`].
mod variable_size_list;

/// Error returned when an [`Array`] cannot be exported.
///
/// Export rejects representation details it cannot preserve faithfully instead
/// of silently changing their meaning, and checks every exported buffer
/// against the array, because a safe [`ExportBuffer`] implementation may
/// borrow as a different slice than the one narrow validated. The enum is
/// non-exhaustive so further unsupported Arrow conditions can be reported
/// explicitly.
///
/// # Examples
///
/// ```
/// use narrow::{array::Array, bitmap::Bitmap, layout::boolean::Boolean};
/// use narrow_ffi::{Export, ExportError};
///
/// let bitmap = Bitmap::try_from_parts(vec![0], 1, 1).unwrap();
/// let array = Array::<bool>::from_buffer(Boolean::from_buffer(bitmap));
/// assert_eq!(array.export().unwrap_err(), ExportError::NonZeroOffset { offset: 1 });
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExportError {
    /// The array has a non-zero offset, which is not currently supported.
    NonZeroOffset {
        /// Unsupported array offset.
        offset: usize,
    },
    /// A buffer holds fewer items than the array requires.
    BufferTooShort {
        /// The buffer that is too short.
        buffer: BufferKind,
        /// Number of items the array requires: bits for bitmaps, offsets for
        /// offset buffers and values otherwise.
        required: usize,
        /// Number of items the buffer holds.
        actual: usize,
    },
    /// The child array of a list holds fewer items than its offsets or its
    /// fixed size require.
    ChildTooShort {
        /// Number of items the list requires, saturated at [`usize::MAX`].
        required: usize,
        /// Number of items the child array holds.
        actual: usize,
    },
    /// The offsets of a list array are negative or decrease.
    ///
    /// The first offset may be greater than zero.
    InvalidOffsets {
        /// Index of the first offset that is negative or smaller than the
        /// offset before it.
        index: usize,
    },
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NonZeroOffset { offset } => {
                write!(f, "array offset ({offset}) is not supported")
            }
            Self::BufferTooShort {
                buffer,
                required,
                actual,
            } => write!(
                f,
                "{buffer} holds {actual} items, but the array requires {required}"
            ),
            Self::ChildTooShort { required, actual } => write!(
                f,
                "child array holds {actual} items, but the list requires {required}"
            ),
            Self::InvalidOffsets { index } => {
                write!(f, "list offset at index {index} is negative or decreasing")
            }
        }
    }
}

impl core::error::Error for ExportError {}

/// Buffer of an exported array, reported by [`ExportError::BufferTooShort`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BufferKind {
    /// The validity bitmap.
    Validity,
    /// The values, or the bits of a boolean array.
    Values,
    /// The offsets of a list array.
    Offsets,
}

impl fmt::Display for BufferKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            Self::Validity => "validity bitmap",
            Self::Values => "values buffer",
            Self::Offsets => "offsets buffer",
        })
    }
}

/// Export an [`Array`] through the Arrow C Data Interface.
///
/// Only arrays with an offset of zero are currently supported.
///
/// Export consumes the array because the returned C handles must retain its
/// storage after Rust leaves this call. The exported array and each of its
/// descendants share ownership of that storage through their `private_data`,
/// so a consumer may move a child array out and release its parent: the
/// storage is dropped once the last of them is released.
///
/// Export borrows each buffer once and checks the borrowed slice against the
/// array, so it is sound for any [`ExportBuffer`], including one whose
/// collections report a length that differs from the slice they borrow as. Buffers may be
/// longer than the array requires. Checking that list offsets do not decrease
/// reads each of them, unless the storage carries a [`Trusted`] token; see
/// [`ExportBuffer`].
///
/// The consumer may read the exported buffers and release the array on any
/// thread, so the array's storage must be [`Send`] and [`Sync`]. Export is only
/// available on targets with atomic pointers (`target_has_atomic = "ptr"`).
///
/// # Examples
///
/// ```
/// use narrow::array::Array;
/// use narrow_ffi::Export;
///
/// let values = [1, 2].into_iter().collect::<Array<i32>>();
/// let (array, schema) = values.export().unwrap();
/// assert!(!array.is_released() && !schema.is_released());
/// ```
///
/// Storage that is not thread-safe, such as
/// [`RcBuffer`](narrow::buffer::RcBuffer), cannot be exported:
///
/// ```compile_fail,E0599
/// use narrow::{array::Array, buffer::RcBuffer};
/// use narrow_ffi::Export;
///
/// let values = [1, 2].into_iter().collect::<Array<i32, RcBuffer>>();
/// let _ = values.export();
/// ```
pub trait Export {
    /// Consumes `self` and returns an [`ArrowArray`] and [`ArrowSchema`].
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::NonZeroOffset`] when the array has a non-zero
    /// offset, [`ExportError::BufferTooShort`] or
    /// [`ExportError::ChildTooShort`] when a buffer or child array holds fewer
    /// items than the array requires, and [`ExportError::InvalidOffsets`] when
    /// the offsets of a list array are negative or decrease.
    ///
    /// # Panics
    ///
    /// Panics when the length of a layout cannot be computed, such as for a
    /// fixed-size list of size zero, or for an [`ExportBuffer`] whose offsets
    /// collection reports no offsets, and when a length or null count exceeds
    /// [`i64::MAX`].
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::array::Array;
    /// use narrow_ffi::Export;
    ///
    /// let values = [1, 2].into_iter().collect::<Array<i32>>();
    /// assert!(values.export().is_ok());
    /// ```
    fn export(self) -> Result<(ArrowArray, ArrowSchema), ExportError>;
}

/// A layout that describes its [`ArrowArray`] fields.
trait ArrowArrayLayout: Length + Sized {
    /// Buffer pointers exposed by the exported array.
    type Buffers: AsRef<[BufferPointer]> + AsMut<[BufferPointer]> + 'static;
    /// Child arrays exposed by the exported array.
    type Children: AsRef<[Exported]> + AsMut<[Exported]> + 'static;

    /// Builds the Arrow schema for this layout.
    fn schema() -> ArrowSchema;

    /// Returns the array's buffer pointers, each taken from a slice that
    /// `checker` checked against the array's length and exported children.
    fn buffers(&self, checker: &mut Checker<'_>) -> Result<Self::Buffers, ExportError>;

    /// Returns the array's child arrays, each sharing ownership of `root`.
    fn children<Root: Send + Sync + 'static>(
        &self,
        root: &Arc<Root>,
    ) -> Result<Self::Children, ExportError>;

    /// Returns the dictionary array pointer.
    fn dictionary(&self) -> *mut ArrowArray {
        ptr::null_mut()
    }

    /// Builds an [`ArrowArray`] for this layout, which is part of `root`.
    ///
    /// The array shares ownership of `root`, so it remains valid when a
    /// consumer moves it out of its parent and releases the parent.
    fn array<Root: Send + Sync + 'static>(&self, root: &Arc<Root>) -> Result<Exported, ExportError>
    where
        Self: 'static,
    {
        checked::array(self, root)
    }

    /// Builds an [`ArrowArray`] and [`ArrowSchema`] from this layout.
    fn export(self) -> Result<(ArrowArray, ArrowSchema), ExportError>
    where
        Self: Send + Sync + 'static,
    {
        // Move the layout to its shared allocation before deriving pointers
        // into its storage. The exported array and all its descendants retain
        // it, so it is dropped once the last of them is released.
        let root = Arc::new(self);
        Ok((root.array(&root)?.into_inner(), Self::schema()))
    }
}

/// Data retained by `ArrowArray::private_data` for an array export.
struct ArrayData<Owner, Layout: ArrowArrayLayout> {
    /// Arrow C Data buffer pointers.
    buffers: Layout::Buffers,
    /// Child arrays owned by the export.
    children: Layout::Children,
    /// Arrow C Data child pointers.
    child_pointers: Vec<*mut ArrowArray>,
    /// Owner retained until the array is released.
    _owner: Owner,
}

impl<Owner: Send + Sync + 'static, Layout: ArrowArrayLayout + 'static> ArrayData<Owner, Layout> {
    /// Creates private data for an exported array.
    fn new(owner: Owner, buffers: Layout::Buffers, children: Layout::Children) -> Self {
        Self {
            buffers,
            children,
            child_pointers: Vec::new(),
            _owner: owner,
        }
    }

    /// Builds an [`ArrowArray`] backed by this private data.
    ///
    /// Pointers into the private data are derived only after it is leaked:
    /// moving the box, or borrowing all of it mutably, would invalidate them.
    fn into_array(
        self: Box<Self>,
        length: i64,
        null_count: i64,
        dictionary: *mut ArrowArray,
    ) -> ArrowArray {
        let private = Box::into_raw(self);
        // SAFETY: `private` comes from `Box::into_raw` and nothing else
        // references it, so these field borrows are unique.
        let buffers = unsafe { (*private).buffers.as_mut() };
        // SAFETY: As above.
        let children = unsafe { (*private).children.as_mut() };
        // SAFETY: As above.
        let child_pointers = unsafe { &mut (*private).child_pointers };
        *child_pointers = children.iter_mut().map(Exported::as_mut_ptr).collect();

        let n_buffers = i64::try_from(buffers.len()).expect("buffer count exceeds i64");
        let n_children = i64::try_from(child_pointers.len()).expect("child count exceeds i64");
        let buffer_pointers = if buffers.is_empty() {
            ptr::null_mut()
        } else {
            // `BufferPointer` is a transparent `*const c_void`.
            buffers.as_mut_ptr().cast()
        };
        let children_pointer = if child_pointers.is_empty() {
            ptr::null_mut()
        } else {
            child_pointers.as_mut_ptr()
        };

        ArrowArray {
            length,
            null_count,
            offset: 0,
            n_buffers,
            n_children,
            buffers: buffer_pointers,
            children: children_pointer,
            dictionary,
            release: Some(release_array::<Self>),
            private_data: private.cast(),
        }
    }
}

impl<T, Storage> Export for Array<T, Storage>
where
    T: ArrayItem,
    Storage: ExportBuffer,
    T::Memory<Storage>: ArrowArrayLayout + Send + Sync + 'static,
{
    fn export(self) -> Result<(ArrowArray, ArrowSchema), ExportError> {
        self.into_buffer().export()
    }
}

impl ArrowSchema {
    /// Builds a childless schema for an [`ArrowType`].
    fn flat<T: ArrowType>() -> Self {
        Self {
            format: T::FORMAT.as_ptr(),
            name: c"".as_ptr(),
            metadata: ptr::null(),
            flags: T::FLAGS,
            n_children: 0,
            children: ptr::null_mut(),
            dictionary: ptr::null_mut(),
            release: Some(release_flat_schema),
            private_data: ptr::null_mut(),
        }
    }
}

/// Releases private data retained by an [`ArrowArray`].
unsafe extern "C" fn release_array<PrivateData>(array: *mut ArrowArray) {
    // SAFETY: The Arrow C Data contract passes the live structure to its
    // producer-provided callback.
    let array_ref = unsafe { &mut *array };
    let private_data = array_ref.private_data;
    array_ref.release = None;
    array_ref.private_data = ptr::null_mut();
    array_ref.buffers = ptr::null_mut();
    array_ref.children = ptr::null_mut();
    array_ref.dictionary = ptr::null_mut();

    // SAFETY: `private_data` was created with `Box::into_raw` for this exact
    // `PrivateData` type and is released only once.
    unsafe { drop(Box::from_raw(private_data.cast::<PrivateData>())) };
}

/// Marks a schema backed only by static data as released.
unsafe extern "C" fn release_flat_schema(schema: *mut ArrowSchema) {
    // SAFETY: The Arrow C Data contract passes the live structure to its
    // producer-provided callback.
    unsafe { (*schema).release = None };
}

/// Releases private data retained by an [`ArrowSchema`].
unsafe extern "C" fn release_schema<PrivateData>(schema: *mut ArrowSchema) {
    // SAFETY: The Arrow C Data contract passes the live structure to its
    // producer-provided callback.
    let schema_ref = unsafe { &mut *schema };
    let private_data = schema_ref.private_data;
    schema_ref.release = None;
    schema_ref.private_data = ptr::null_mut();
    schema_ref.children = ptr::null_mut();
    schema_ref.dictionary = ptr::null_mut();

    // SAFETY: `private_data` was created with `Box::into_raw` for this exact
    // `PrivateData` type and is released only once.
    unsafe { drop(Box::from_raw(private_data.cast::<PrivateData>())) };
}

/// Moves the first child out of `parent` as the C Data Interface allows: the
/// child is copied and the original is marked released.
#[cfg(test)]
fn move_first_child(parent: &ArrowArray) -> ArrowArray {
    assert!(parent.n_children > 0, "array has no children");
    // SAFETY: A live exported array with children has a valid child pointer array.
    let child = unsafe { *parent.children };
    // SAFETY: The child pointer refers to a live array owned by the parent.
    unsafe { ptr::replace(child, ArrowArray::default()) }
}

/// Reads a `list<int32>` array as a consumer that follows the Arrow C Data
/// Interface does.
#[cfg(test)]
fn read_list_i32(array: &ArrowArray) -> Vec<Vec<i32>> {
    use core::slice;

    let length = usize::try_from(array.length).expect("non-negative length");
    // SAFETY: An exported list array has two buffers.
    let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
    // SAFETY: The offsets buffer holds `length + 1` offsets.
    let offsets = unsafe { slice::from_raw_parts(buffers[1].cast::<i32>(), length.strict_add(1)) };
    // SAFETY: An exported list array has one child.
    let child_pointer = unsafe { *array.children };
    // SAFETY: The child pointer refers to a live array owned by the parent.
    let child = unsafe { &*child_pointer };
    let child_length = usize::try_from(child.length).expect("non-negative length");
    // SAFETY: An exported primitive array has two buffers.
    let child_buffers = unsafe { slice::from_raw_parts(child.buffers, 2) };
    // SAFETY: The values buffer of the child holds `child_length` values.
    let values = unsafe { slice::from_raw_parts(child_buffers[1].cast::<i32>(), child_length) };
    offsets
        .windows(2)
        .map(|pair| {
            let start = usize::try_from(pair[0]).expect("non-negative offset");
            let end = usize::try_from(pair[1]).expect("non-negative offset");
            values[start..end].to_vec()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    extern crate std;

    use alloc::{sync::Arc, vec::Vec};
    use core::slice;
    use std::{
        panic::{AssertUnwindSafe, catch_unwind},
        thread,
    };

    use narrow::{
        array::Array,
        buffer::ArcBuffer,
        collection::{Collection, owned::IntoOwned},
        layout::{
            ArrayItem, fixed_size_primitive::FixedSizePrimitive,
            variable_size_list::VariableSizeList,
        },
        offset::Offsets,
    };

    use super::Export;

    /// Exports `array` after an `extend` that panicked part way, which leaves
    /// its buffers longer than the array, and returns its length and null
    /// count.
    fn export_after_interrupted_extend<T>(mut array: Array<Option<T>>, item: T) -> (i64, i64)
    where
        T: Clone,
        Option<T>: ArrayItem,
        Array<Option<T>>: Extend<Option<T>> + Export,
    {
        let interrupted = catch_unwind(AssertUnwindSafe(|| {
            array.extend((0..4).map(|index| {
                assert!(index < 3, "interrupt extend");
                Some(item.clone())
            }));
        }));
        assert!(interrupted.is_err());
        let (exported, _schema) = array.export().expect("export array");
        (exported.length, exported.null_count)
    }

    /// Extends `array` with three copies of `item` and then panics, which
    /// leaves the buffers of nullable arrays longer than the array.
    fn interrupt_extend<T: Clone>(array: &mut impl Extend<T>, item: &T) {
        let interrupted = catch_unwind(AssertUnwindSafe(|| {
            array.extend((0..4).map(|index| {
                assert!(index < 3, "interrupt extend");
                item.clone()
            }));
        }));
        assert!(interrupted.is_err());
    }

    #[test]
    fn exports_non_nullable_after_interrupted_extend() {
        let mut primitive = [1].into_iter().collect::<Array<i32>>();
        interrupt_extend(&mut primitive, &2);
        let (primitive_array, _primitive_schema) = primitive.export().expect("export array");
        assert_eq!(primitive_array.length, 4);

        let mut list = [Vec::from([1])].into_iter().collect::<Array<Vec<i32>>>();
        interrupt_extend(&mut list, &Vec::from([2]));
        let expected: Vec<Vec<i32>> = list.iter_views().map(IntoOwned::into_owned).collect();
        let (list_array, _list_schema) = list.export().expect("export array");
        assert_eq!(super::read_list_i32(&list_array), expected);

        let mut fixed_size_list = [[1, 2]].into_iter().collect::<Array<[i32; 2]>>();
        interrupt_extend(&mut fixed_size_list, &[3, 4]);
        let (fixed_size_list_array, _fixed_size_list_schema) =
            fixed_size_list.export().expect("export array");
        assert_eq!(fixed_size_list_array.length, 4);
    }

    #[test]
    fn exports_after_interrupted_extend() {
        let primitive = [Some(1), None].into_iter().collect();
        assert_eq!(export_after_interrupted_extend(primitive, 2), (2, 1));

        let list = [Some(Vec::from([1])), None].into_iter().collect();
        assert_eq!(
            export_after_interrupted_extend(list, Vec::from([2])),
            (2, 1)
        );

        let fixed_size_list = [Some([1, 2]), None].into_iter().collect();
        assert_eq!(
            export_after_interrupted_extend(fixed_size_list, [3, 4]),
            (2, 1)
        );
    }

    #[test]
    fn releases_exported_array_on_another_thread() {
        let value_storage = Arc::<[i32]>::from([1, 2, 3]);
        let retained = Arc::clone(&value_storage);
        let values_layout = FixedSizePrimitive::from_buffer(value_storage);
        let offsets =
            Offsets::try_from_parts(values_layout, Arc::from([0, 2, 3])).expect("valid offsets");
        let narrow_array: Array<Vec<i32>, ArcBuffer> =
            Array::from_buffer(VariableSizeList::from_buffer(offsets));

        let (array, schema) = narrow_array.export().expect("export array");
        let worker = thread::spawn(move || {
            // SAFETY: The exported array owns a one-entry child pointer array.
            let children = unsafe { slice::from_raw_parts(array.children, 1) };
            // SAFETY: The child pointer refers to an array retained by the parent.
            let child = unsafe { &*children[0] };
            // SAFETY: The child owns a two-entry buffer pointer array.
            let buffers = unsafe { slice::from_raw_parts(child.buffers, 2) };
            assert_eq!(
                // SAFETY: The value buffer points to three values retained by the parent.
                unsafe { slice::from_raw_parts(buffers[1].cast::<i32>(), 3) },
                [1, 2, 3]
            );
            drop((array, schema));
        });
        for _ in 0..100 {
            drop(Arc::clone(&retained));
        }
        worker.join().expect("release on worker thread");

        assert_eq!(Arc::strong_count(&retained), 1);
    }
}
