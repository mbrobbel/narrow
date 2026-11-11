//! Export [`Array`] values through the Arrow C Data Interface.

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::{
    ffi::{CStr, c_void},
    fmt, ptr,
};

use narrow::{
    array::Array,
    buffer::{ArcBuffer, ArrayBuffer, BoxBuffer, Buffer, RcBuffer, SliceBuffer, VecBuffer},
    layout::ArrayItem,
    length::Length,
};

use crate::{ARROW_FLAG_NULLABLE, ArrowArray, ArrowSchema};

/// Export support for [`narrow::layout::boolean::Boolean`].
mod boolean;
/// Export support for [`narrow::layout::fixed_size_list::FixedSizeList`].
mod fixed_size_list;
/// Export support for [`narrow::layout::fixed_size_primitive::FixedSizePrimitive`].
mod fixed_size_primitive;
/// Export support for [`narrow::layout::variable_size_list::VariableSizeList`].
mod variable_size_list;

/// A type with an [Arrow C Data format string].
///
/// [Arrow C Data format string]: https://arrow.apache.org/docs/format/CDataInterface.html#data-type-description-format-strings
///
/// The Rust item type already determines its Arrow layout. These associated
/// constants carry the corresponding C schema description at the type level,
/// including the nullable flag added by `Option<T>`.
///
/// # Examples
///
/// ```
/// use narrow_ffi::{ARROW_FLAG_NULLABLE, ArrowType};
///
/// assert_eq!(i32::FORMAT, c"i");
/// assert_eq!(<Option<i32>>::FLAGS, ARROW_FLAG_NULLABLE);
/// ```
pub trait ArrowType {
    /// Arrow C Data type format.
    const FORMAT: &'static CStr;
    /// Arrow C Data schema flags.
    const FLAGS: i64 = 0;
}

impl<T: ArrowType> ArrowType for Option<T> {
    const FORMAT: &'static CStr = T::FORMAT;
    const FLAGS: i64 = T::FLAGS | ARROW_FLAG_NULLABLE;
}

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
/// storage after Rust leaves this call. That storage and the exposed pointer
/// tables live in `private_data` until the foreign consumer invokes the Arrow
/// release callback.
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

    /// Returns the array's child pointers.
    fn children(&self) -> Result<Self::Children, ExportError> {
        Ok(Self::Children::default())
    }

    /// Returns the dictionary array pointer.
    fn dictionary(&self) -> *mut ArrowArray {
        ptr::null_mut()
    }

    /// Builds a child [`ArrowArray`] borrowing storage retained by its parent.
    fn child_array(&self) -> Result<ArrowArray, ExportError>
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
        let private = Box::new(ArrayData::<(), Self>::new(
            (),
            self.buffers(),
            self.children()?,
        ));

        Ok(private.into_array(length, null_count, dictionary))
    }

    /// Builds an [`ArrowArray`] and [`ArrowSchema`] from this layout.
    fn export(self) -> Result<(ArrowArray, ArrowSchema), ExportError>
    where
        Self: 'static,
    {
        let offset = self.offset();
        if offset != 0 {
            return Err(ExportError::NonZeroOffset { offset });
        }

        // Pin the layout before asking it for pointers into its storage. The
        // box is leaked first because moving it, or borrowing all of it
        // mutably, would invalidate pointers derived from the owner.
        let private = Box::into_raw(Box::new(ArrayData::<Self, Self>::new(
            self,
            Self::Buffers::default(),
            Self::Children::default(),
        )));
        // SAFETY: `private` comes from `Box::into_raw` and is valid. This
        // borrows only the owner, which is never written to again, so the
        // writes to other fields below keep pointers derived from it valid.
        let owner = unsafe { &(*private).owner };
        let children = match owner.children() {
            Ok(children) => children,
            Err(error) => {
                // SAFETY: Reclaims the box leaked above, which nothing else
                // references.
                drop(unsafe { Box::from_raw(private) });
                return Err(error);
            }
        };
        let buffers = owner.buffers();

        // Convert platform-sized layout metadata into the Arrow C ABI fields.
        let length = i64::try_from(owner.len()).expect("array length exceeds i64");
        let null_count = owner.null_count();
        let dictionary = owner.dictionary();

        // SAFETY: `private` is valid and these fields are disjoint from the
        // borrowed owner.
        unsafe { (*private).buffers = buffers };
        // SAFETY: As above.
        unsafe { (*private).children = children };
        // SAFETY: `private` comes from `Box::into_raw` and its owner is only
        // borrowed shared.
        let array =
            unsafe { ArrayData::leaked_into_array(private, length, null_count, dictionary) };

        Ok((array, Self::schema()))
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
    owner: Owner,
}

impl<Owner: 'static, Layout: ArrowArrayLayout + 'static> ArrayData<Owner, Layout> {
    /// Creates private data for an exported array.
    fn new(owner: Owner, buffers: Layout::Buffers, children: Layout::Children) -> Self {
        Self {
            buffers,
            children,
            child_pointers: Vec::new(),
            owner,
        }
    }

    /// Builds an [`ArrowArray`] backed by this private data.
    fn into_array(
        self: Box<Self>,
        length: i64,
        null_count: i64,
        dictionary: *mut ArrowArray,
    ) -> ArrowArray {
        // SAFETY: The pointer comes from `Box::into_raw` and nothing else
        // references it.
        unsafe { Self::leaked_into_array(Box::into_raw(self), length, null_count, dictionary) }
    }

    /// Builds an [`ArrowArray`] backed by leaked private data, taking
    /// ownership of it.
    ///
    /// Pointers into the private data are derived only after it is leaked:
    /// moving the box, or borrowing all of it mutably, would invalidate them.
    ///
    /// # Safety
    ///
    /// `private` must come from [`Box::into_raw`], and no reference to its
    /// fields other than shared references to the owner may be live.
    unsafe fn leaked_into_array(
        private: *mut Self,
        length: i64,
        null_count: i64,
        dictionary: *mut ArrowArray,
    ) -> ArrowArray {
        // SAFETY: The caller guarantees `private` is valid and that only the
        // owner may be borrowed, so these field borrows are unique.
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
    T::Memory<Storage>: ArrowArrayLayout + 'static,
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
