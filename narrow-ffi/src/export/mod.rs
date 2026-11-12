//! Export [`Array`] values through the Arrow C Data Interface.

extern crate alloc;

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{ffi::c_void, fmt, ptr};

use narrow::{
    array::Array,
    buffer::{ArcBuffer, ArrayBuffer, BoxBuffer, Buffer, RcBuffer, SliceBuffer, VecBuffer},
    layout::ArrayItem,
    length::Length,
};

use crate::{ArrowArray, ArrowSchema, ArrowType};

/// Export support for [`narrow::layout::boolean::Boolean`].
mod boolean;
/// Export support for [`narrow::layout::fixed_size_list::FixedSizeList`].
mod fixed_size_list;
/// Export support for [`narrow::layout::fixed_size_primitive::FixedSizePrimitive`].
mod fixed_size_primitive;
/// Export support for [`narrow::layout::variable_size_list::VariableSizeList`].
mod variable_size_list;

/// Error returned when an [`Array`] cannot be exported.
///
/// Export rejects representation details it cannot preserve faithfully instead
/// of silently changing their meaning. The enum is non-exhaustive so further
/// unsupported Arrow conditions can be reported explicitly.
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
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NonZeroOffset { offset } => {
                write!(f, "array offset ({offset}) is not supported")
            }
        }
    }
}

impl core::error::Error for ExportError {}

/// Export an [`Array`] through the Arrow C Data Interface.
///
/// Only arrays with an offset of zero are currently supported, and their
/// storage must implement [`TrustedBuffer`].
///
/// Export consumes the array because the returned C handles must retain its
/// storage after Rust leaves this call. The exported array and each of its
/// descendants share ownership of that storage through their `private_data`,
/// so a consumer may move a child array out and release its parent: the
/// storage is dropped once the last of them is released.
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
    /// offset.
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

/// [`Buffer`] storage whose collections can be exported without copying.
///
/// An exported [`ArrowArray`] pairs pointers into its buffers with lengths that
/// narrow records when it builds or validates a layout. Foreign consumers read
/// memory based on both, so the buffers must hold exactly the items narrow
/// recorded, which a safe [`Buffer`] implementation cannot promise. [`Export`]
/// is therefore implemented only for arrays whose storage implements this
/// trait. All storages provided by [`narrow::buffer`] implement it.
///
/// # Safety
///
/// For every `T: FixedSize`, `Self::For<T>` must behave exactly like the
/// sequence of `T` in the slice it borrows as:
///
/// - [`Length::len`] and the [`Collection`] items must describe the slice
///   returned by [`Borrow::<[T]>::borrow`](core::borrow::Borrow::borrow), and
///   [`BorrowMut::<[T]>::borrow_mut`](core::borrow::BorrowMut::borrow_mut), if
///   implemented, must return that same slice.
/// - The memory of a borrowed slice must stay valid until the value is moved,
///   accessed through `&mut` or dropped.
/// - The length and items of the slice may only change through operations on
///   the value through `&mut`, and only as those operations describe. Moving
///   the value may change the address of the slice, but not its length or
///   items.
/// - Implementations that create, copy or extend values, such as [`Default`],
///   [`Clone`], [`FromIterator`], [`Extend`] and those of [`CollectionAllocIn`]
///   and [`CollectionRealloc`], must produce exactly the items they describe:
///   the same items for `Clone`, the given items, in order, when collecting or
///   extending, and no items for `with_capacity_in`, `try_with_capacity_in`
///   or, if `Self::For<T>` implements `CollectionRealloc`, `Default`.
///
/// [`Collection`]: narrow::collection::Collection
/// [`CollectionAllocIn`]: narrow::collection::CollectionAllocIn
/// [`CollectionRealloc`]: narrow::collection::CollectionRealloc
///
/// # Examples
///
/// Storage backed by [`Vec`] can be exported once it implements this trait:
///
/// ```
/// use narrow::{array::Array, buffer::Buffer, fixed_size::FixedSize};
/// use narrow_ffi::{Export, TrustedBuffer};
///
/// #[derive(Default)]
/// struct MyBuffer;
///
/// impl Buffer for MyBuffer {
///     type For<T: FixedSize> = Vec<T>;
/// }
///
/// // SAFETY: A `Vec` behaves exactly like the slice it borrows as.
/// unsafe impl TrustedBuffer for MyBuffer {}
///
/// let array = [1, 2].into_iter().collect::<Array<i32, MyBuffer>>();
/// assert!(array.export().is_ok());
/// ```
///
/// Storage that does not implement this trait cannot be exported, so a safe
/// collection that reports more items than its slice holds is rejected:
///
/// ```compile_fail,E0599
/// use std::borrow::Borrow;
///
/// use narrow::{
///     array::Array, buffer::Buffer, collection::Collection, fixed_size::FixedSize,
///     layout::fixed_size_primitive::FixedSizePrimitive, length::Length,
/// };
/// use narrow_ffi::Export;
///
/// struct Overreported<T>(Vec<T>);
///
/// impl<T> Length for Overreported<T> {
///     fn len(&self) -> usize {
///         self.0.len() + 1
///     }
/// }
///
/// impl<T> Borrow<[T]> for Overreported<T> {
///     fn borrow(&self) -> &[T] {
///         &self.0
///     }
/// }
///
/// # impl<T: FixedSize> Collection for Overreported<T> {
/// #     type View<'collection> = T where Self: 'collection;
/// #     type Owned = T;
/// #     fn view(&self, index: usize) -> Option<T> {
/// #         self.0.get(index).copied()
/// #     }
/// #     type Iter<'collection> = std::iter::Copied<std::slice::Iter<'collection, T>>
/// #     where
/// #         Self: 'collection;
/// #     fn iter_views(&self) -> Self::Iter<'_> {
/// #         self.0.iter().copied()
/// #     }
/// #     type IntoIter = std::vec::IntoIter<T>;
/// #     fn into_iter_owned(self) -> Self::IntoIter {
/// #         self.0.into_iter()
/// #     }
/// # }
/// #
/// #[derive(Default)]
/// struct Overreporting;
///
/// impl Buffer for Overreporting {
///     type For<T: FixedSize> = Overreported<T>;
/// }
///
/// let array: Array<i32, Overreporting> =
///     Array::from_buffer(FixedSizePrimitive::from_buffer(Overreported(vec![1, 2, 3])));
/// let _ = array.export();
/// ```
pub unsafe trait TrustedBuffer: Buffer {}

// SAFETY: A `Vec` behaves exactly like the slice of initialized items it
// borrows as, and its heap allocation only changes through `&mut`.
unsafe impl TrustedBuffer for VecBuffer {}

// SAFETY: A boxed slice behaves exactly like the slice it borrows as, and its
// heap allocation only changes through `&mut`.
unsafe impl TrustedBuffer for BoxBuffer {}

// SAFETY: A shared slice behaves exactly like the slice it borrows as, and its
// heap allocation only changes through `&mut`, as `Rc::get_mut` requires.
unsafe impl TrustedBuffer for RcBuffer {}

// SAFETY: As for `RcBuffer`, with `Arc::get_mut`.
unsafe impl TrustedBuffer for ArcBuffer {}

// SAFETY: An array behaves exactly like the slice it borrows as and does not
// implement `CollectionRealloc`, so its `Default` may hold items. Its inline
// items only change through `&mut`; moving it only changes their address.
unsafe impl<const N: usize> TrustedBuffer for ArrayBuffer<N> {}

// SAFETY: A slice reference is the slice it borrows as, and the memory it
// refers to is immutable for its lifetime.
unsafe impl TrustedBuffer for SliceBuffer<'_> {}

/// A layout that describes its [`ArrowArray`] fields.
trait ArrowArrayLayout: Length + Sized {
    /// Buffer pointers exposed by the exported array.
    type Buffers: AsRef<[*const c_void]> + AsMut<[*const c_void]> + Default + 'static;
    /// Child arrays exposed by the exported array.
    type Children: AsRef<[ArrowArray]> + AsMut<[ArrowArray]> + Default + 'static;

    /// Builds the Arrow schema for this layout.
    fn schema() -> ArrowSchema;

    /// Returns the number of null elements, or `-1` when unknown.
    fn null_count(&self) -> i64 {
        0
    }

    /// Returns the item offset into the buffers.
    fn offset(&self) -> usize {
        0
    }

    /// Returns the array's buffer pointers.
    fn buffers(&self) -> Self::Buffers;

    /// Returns the array's child arrays, each sharing ownership of `root`.
    fn children<Root: Send + Sync + 'static>(
        &self,
        _root: &Arc<Root>,
    ) -> Result<Self::Children, ExportError> {
        Ok(Self::Children::default())
    }

    /// Returns the dictionary array pointer.
    fn dictionary(&self) -> *mut ArrowArray {
        ptr::null_mut()
    }

    /// Builds an [`ArrowArray`] for this layout, which is part of `root`.
    ///
    /// The array shares ownership of `root`, so it remains valid when a
    /// consumer moves it out of its parent and releases the parent.
    fn array<Root: Send + Sync + 'static>(
        &self,
        root: &Arc<Root>,
    ) -> Result<ArrowArray, ExportError>
    where
        Self: 'static,
    {
        let offset = self.offset();
        if offset != 0 {
            return Err(ExportError::NonZeroOffset { offset });
        }

        let length = i64::try_from(self.len()).expect("array length exceeds i64");
        let null_count = self.null_count();
        let dictionary = self.dictionary();
        let private = Box::new(ArrayData::<Arc<Root>, Self>::new(
            Arc::clone(root),
            self.buffers(),
            self.children(root)?,
        ));

        Ok(private.into_array(length, null_count, dictionary))
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
        Ok((root.array(&root)?, Self::schema()))
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
        *child_pointers = children.iter_mut().map(ptr::from_mut).collect();

        let n_buffers = i64::try_from(buffers.len()).expect("buffer count exceeds i64");
        let n_children = i64::try_from(child_pointers.len()).expect("child count exceeds i64");
        let buffer_pointers = if buffers.is_empty() {
            ptr::null_mut()
        } else {
            buffers.as_mut_ptr()
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
    Storage: TrustedBuffer,
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

#[cfg(test)]
mod tests {
    extern crate alloc;
    extern crate std;

    use alloc::{sync::Arc, vec::Vec};
    use core::slice;
    use std::thread;

    use narrow::{
        array::Array,
        buffer::ArcBuffer,
        layout::{fixed_size_primitive::FixedSizePrimitive, variable_size_list::VariableSizeList},
        offset::Offsets,
    };

    use super::Export;

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
