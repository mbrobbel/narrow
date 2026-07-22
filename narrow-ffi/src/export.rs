//! Export [`Array`] values through the Arrow C Data Interface.

extern crate alloc;

use alloc::boxed::Box;
use core::{
    borrow::Borrow,
    ffi::{CStr, c_void},
    fmt, ptr,
};

use narrow::{
    array::Array,
    bitmap::{BitmapRef, ValidityBitmap},
    buffer::{Buffer, BufferRef},
    collection::ChildRef,
    fixed_size::FixedSize,
    layout::{ArrayItem, boolean::Boolean, fixed_size_primitive::FixedSizePrimitive},
    length::Length,
    nullability::{NonNullable, Nullable},
};

use crate::{ARROW_FLAG_NULLABLE, ArrowArray, ArrowSchema};

/// A type with an [Arrow C Data format string].
///
/// [Arrow C Data format string]: https://arrow.apache.org/docs/format/CDataInterface.html#data-type-description-format-strings
///
/// # Design
///
/// Narrow's Rust item type already determines its Arrow layout. These
/// associated constants carry the corresponding C schema description at the
/// type level, including the nullable flag added by `Option<T>`.
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

impl ArrowType for bool {
    const FORMAT: &'static CStr = c"b";
}

impl ArrowType for i8 {
    const FORMAT: &'static CStr = c"c";
}

impl ArrowType for u8 {
    const FORMAT: &'static CStr = c"C";
}

impl ArrowType for i16 {
    const FORMAT: &'static CStr = c"s";
}

impl ArrowType for u16 {
    const FORMAT: &'static CStr = c"S";
}

impl ArrowType for i32 {
    const FORMAT: &'static CStr = c"i";
}

impl ArrowType for u32 {
    const FORMAT: &'static CStr = c"I";
}

impl ArrowType for i64 {
    const FORMAT: &'static CStr = c"l";
}

impl ArrowType for u64 {
    const FORMAT: &'static CStr = c"L";
}

impl ArrowType for f32 {
    const FORMAT: &'static CStr = c"f";
}

impl ArrowType for f64 {
    const FORMAT: &'static CStr = c"g";
}

/// Error returned when an [`Array`] cannot be exported.
///
/// # Design
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
        match self {
            Self::NonZeroOffset { offset } => {
                write!(f, "array offset ({offset}) is not supported")
            }
        }
    }
}

impl core::error::Error for ExportError {}

/// Export an [`Array`] through the Arrow C Data Interface.
///
/// Only arrays with an offset of zero are currently supported.
///
/// # Design
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

/// A layout that describes its [`ArrowArray`] fields.
trait ArrowArrayLayout: Length + Sized {
    /// Buffer pointers exposed by the exported array.
    type Buffers: AsRef<[*const c_void]> + AsMut<[*const c_void]> + Default + 'static;
    /// Child pointers exposed by the exported array.
    type Children: AsRef<[*mut ArrowArray]> + AsMut<[*mut ArrowArray]> + Default + 'static;

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
    fn children(&mut self) -> Self::Children {
        Self::Children::default()
    }
    /// Returns the dictionary array pointer.
    fn dictionary(&mut self) -> *mut ArrowArray {
        ptr::null_mut()
    }

    /// Builds an [`ArrowArray`] and [`ArrowSchema`] from this layout.
    fn export<T: ArrowType>(self) -> Result<(ArrowArray, ArrowSchema), ExportError>
    where
        Self: 'static,
    {
        let offset = self.offset();
        if offset != 0 {
            return Err(ExportError::NonZeroOffset { offset });
        }

        // Pin the layout before asking it for pointers into its storage.
        let mut private = Box::new(ArrayData::<Self> {
            layout: self,
            buffers: Self::Buffers::default(),
            children: Self::Children::default(),
        });
        private.buffers = private.layout.buffers();
        private.children = private.layout.children();

        // Convert platform-sized layout metadata into the Arrow C ABI fields.
        let length = i64::try_from(private.layout.len()).expect("array length exceeds i64");
        let null_count = private.layout.null_count();
        let n_buffers =
            i64::try_from(private.buffers.as_ref().len()).expect("buffer count exceeds i64");
        let n_children =
            i64::try_from(private.children.as_ref().len()).expect("child count exceeds i64");
        let dictionary = private.layout.dictionary();

        // Keep the layout data and pointer collections alive until release.
        let buffers = private.buffers.as_mut();
        let buffers = if buffers.is_empty() {
            ptr::null_mut()
        } else {
            buffers.as_mut_ptr()
        };
        let children = private.children.as_mut();
        let children = if children.is_empty() {
            ptr::null_mut()
        } else {
            children.as_mut_ptr()
        };
        let private_data = Box::into_raw(private).cast();
        let array = ArrowArray {
            length,
            null_count,
            offset: 0,
            n_buffers,
            n_children,
            buffers,
            children,
            dictionary,
            release: Some(release_array::<ArrayData<Self>>),
            private_data,
        };

        Ok((array, ArrowSchema::flat::<T>()))
    }
}

impl<T, Storage> ArrowArrayLayout for FixedSizePrimitive<T, NonNullable, Storage>
where
    T: FixedSize + ArrowType,
    Storage: Buffer,
{
    type Buffers = [*const c_void; 2];
    type Children = [*mut ArrowArray; 0];

    fn buffers(&self) -> Self::Buffers {
        let values: &[T] = self.buffer_ref().borrow();
        [ptr::null(), values.as_ptr().cast()]
    }
}

impl<T, Storage> ArrowArrayLayout for FixedSizePrimitive<T, Nullable, Storage>
where
    T: FixedSize + ArrowType,
    Storage: Buffer,
{
    type Buffers = [*const c_void; 2];
    type Children = [*mut ArrowArray; 0];

    fn null_count(&self) -> i64 {
        i64::try_from(ValidityBitmap::null_count(self.buffer_ref()))
            .expect("null count exceeds i64")
    }

    fn offset(&self) -> usize {
        self.buffer_ref().bitmap_ref().bit_offset()
    }

    fn buffers(&self) -> Self::Buffers {
        let validity = self.buffer_ref();
        let validity_values: &[u8] = validity.bitmap_ref().buffer_ref().borrow();
        let values: &[T] = validity.child_ref().borrow();
        [validity_values.as_ptr().cast(), values.as_ptr().cast()]
    }
}

impl<Storage: Buffer> ArrowArrayLayout for Boolean<NonNullable, Storage> {
    type Buffers = [*const c_void; 2];
    type Children = [*mut ArrowArray; 0];

    fn offset(&self) -> usize {
        self.buffer_ref().bit_offset()
    }

    fn buffers(&self) -> Self::Buffers {
        let values: &[u8] = self.buffer_ref().buffer_ref().borrow();
        [ptr::null(), values.as_ptr().cast()]
    }
}

/// Data retained by `ArrowArray::private_data` for an array export.
struct ArrayData<Layout: ArrowArrayLayout> {
    /// Layout backing the exported array.
    layout: Layout,
    /// Arrow C Data buffer pointers.
    buffers: Layout::Buffers,
    /// Arrow C Data child pointers.
    children: Layout::Children,
}

impl<T, Storage> Export for Array<T, Storage>
where
    T: ArrayItem + ArrowType,
    Storage: Buffer,
    T::Memory<Storage>: ArrowArrayLayout + 'static,
{
    fn export(self) -> Result<(ArrowArray, ArrowSchema), ExportError> {
        self.into_buffer().export::<T>()
    }
}

impl ArrowSchema {
    fn flat<T: ArrowType>() -> Self {
        Self {
            format: T::FORMAT.as_ptr(),
            name: c"".as_ptr(),
            metadata: ptr::null(),
            flags: T::FLAGS,
            n_children: 0,
            children: ptr::null_mut(),
            dictionary: ptr::null_mut(),
            release: Some(release_schema),
            private_data: ptr::null_mut(),
        }
    }
}

unsafe extern "C" fn release_array<PrivateData>(array: *mut ArrowArray) {
    // SAFETY: The Arrow C Data contract passes the live structure to its
    // producer-provided callback.
    let array = unsafe { &mut *array };
    let private_data = array.private_data;
    array.release = None;
    array.private_data = ptr::null_mut();
    array.buffers = ptr::null_mut();
    array.children = ptr::null_mut();
    array.dictionary = ptr::null_mut();

    // SAFETY: `private_data` was created with `Box::into_raw` for this exact
    // `PrivateData` type and is released only once.
    unsafe { drop(Box::from_raw(private_data.cast::<PrivateData>())) };
}

unsafe extern "C" fn release_schema(schema: *mut ArrowSchema) {
    // SAFETY: The Arrow C Data contract passes the live structure to its
    // producer-provided callback.
    unsafe { (*schema).release = None };
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::sync::Arc;
    use core::{ffi::CStr, slice};

    use crate::ARROW_FLAG_NULLABLE;

    use narrow::{
        array::Array,
        bitmap::Bitmap,
        buffer::{ArcBuffer, ArrayBuffer},
        layout::{boolean::Boolean, fixed_size_primitive::FixedSizePrimitive},
        validity::Validity,
    };

    use super::{ArrowType, Export, ExportError};

    #[test]
    fn type_format_strings_match_arrow() {
        assert_eq!(bool::FORMAT, c"b");
        assert_eq!(i8::FORMAT, c"c");
        assert_eq!(u8::FORMAT, c"C");
        assert_eq!(i16::FORMAT, c"s");
        assert_eq!(u16::FORMAT, c"S");
        assert_eq!(i32::FORMAT, c"i");
        assert_eq!(u32::FORMAT, c"I");
        assert_eq!(i64::FORMAT, c"l");
        assert_eq!(u64::FORMAT, c"L");
        assert_eq!(f32::FORMAT, c"f");
        assert_eq!(f64::FORMAT, c"g");
        assert_eq!(<Option<i32>>::FORMAT, c"i");
        assert_eq!(<Option<i32>>::FLAGS, ARROW_FLAG_NULLABLE);
    }

    #[test]
    fn exports_primitive_array_without_copying_values() {
        let values = Arc::<[i32]>::from([1, 2, 3]);
        let weak = Arc::downgrade(&values);
        let data = values.as_ptr();
        let array: Array<i32, ArcBuffer> =
            Array::from_buffer(FixedSizePrimitive::from_buffer(values));

        let (array, schema) = array.export().expect("export array");

        assert_eq!(array.length, 3);
        assert_eq!(array.null_count, 0);
        assert_eq!(array.offset, 0);
        assert_eq!(array.n_buffers, 2);
        assert_eq!(array.n_children, 0);
        assert!(array.children.is_null());
        assert!(array.dictionary.is_null());
        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        assert!(buffers[0].is_null());
        assert_eq!(buffers[1], data.cast());
        // SAFETY: The second buffer points to three i32 values held by the export.
        assert_eq!(
            unsafe { slice::from_raw_parts(buffers[1].cast::<i32>(), 3) },
            [1, 2, 3]
        );

        // SAFETY: The exported schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(schema.format) }, c"i");
        assert_eq!(schema.flags, 0);
        assert_eq!(schema.n_children, 0);
        assert!(schema.children.is_null());
        assert!(schema.dictionary.is_null());
        assert!(weak.upgrade().is_some());

        drop(array);
        assert!(weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn pins_inline_values_for_export() {
        let array: Array<i32, ArrayBuffer<3>> =
            Array::from_buffer(FixedSizePrimitive::from_buffer([1, 2, 3]));

        let (array, _schema) = array.export().expect("export array");

        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        // SAFETY: The second buffer points to three inline i32 values pinned in
        // the export's private allocation.
        assert_eq!(
            unsafe { slice::from_raw_parts(buffers[1].cast::<i32>(), 3) },
            [1, 2, 3]
        );
    }

    #[test]
    fn exports_nullable_primitive_buffers_without_copying() {
        let values = Arc::<[i32]>::from([1, 0, 3]);
        let validity_values = Arc::<[u8]>::from([0b0000_0101]);
        let values_weak = Arc::downgrade(&values);
        let validity_weak = Arc::downgrade(&validity_values);
        let values_data = values.as_ptr();
        let validity_data = validity_values.as_ptr();
        let bitmap =
            Bitmap::<ArcBuffer>::try_from_parts(validity_values, 3, 0).expect("valid bitmap");
        let validity = Validity::try_from_parts(values, bitmap).expect("valid parts");
        let array: Array<Option<i32>, ArcBuffer> =
            Array::from_buffer(FixedSizePrimitive::from_buffer(validity));

        let (array, schema) = array.export().expect("export array");

        assert_eq!(array.length, 3);
        assert_eq!(array.null_count, 1);
        assert_eq!(array.offset, 0);
        assert_eq!(array.n_buffers, 2);
        assert_eq!(array.n_children, 0);
        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        assert_eq!(buffers[0], validity_data.cast());
        assert_eq!(buffers[1], values_data.cast());
        // SAFETY: The buffers point to storage retained by the export.
        assert_eq!(unsafe { *buffers[0].cast::<u8>() }, 0b0000_0101);
        // SAFETY: The value buffer points to three i32 values.
        assert_eq!(
            unsafe { slice::from_raw_parts(buffers[1].cast::<i32>(), 3) },
            [1, 0, 3]
        );

        // SAFETY: The exported schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(schema.format) }, c"i");
        assert_eq!(schema.flags, ARROW_FLAG_NULLABLE);
        assert!(values_weak.upgrade().is_some());
        assert!(validity_weak.upgrade().is_some());

        drop(array);
        assert!(values_weak.upgrade().is_none());
        assert!(validity_weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn rejects_non_zero_nullable_primitive_offset() {
        let bitmap = Bitmap::<ArrayBuffer<3>>::try_from_parts([0b0001_0100, 0, 0], 3, 2)
            .expect("valid bitmap");
        let validity = Validity::try_from_parts([1, 0, 3], bitmap).expect("valid parts");
        let array: Array<Option<i32>, ArrayBuffer<3>> =
            Array::from_buffer(FixedSizePrimitive::from_buffer(validity));

        let error = array.export().expect_err("non-zero offset");

        assert_eq!(error, ExportError::NonZeroOffset { offset: 2 });
    }

    #[test]
    fn exports_boolean_bitmap_without_copying_values() {
        let values = Arc::<[u8]>::from([0b0000_0101]);
        let weak = Arc::downgrade(&values);
        let data = values.as_ptr();
        let bitmap = Bitmap::<ArcBuffer>::try_from_parts(values, 3, 0).expect("valid bitmap");
        let array: Array<bool, ArcBuffer> = Array::from_buffer(Boolean::from_buffer(bitmap));

        let (array, schema) = array.export().expect("export array");

        assert_eq!(array.length, 3);
        assert_eq!(array.null_count, 0);
        assert_eq!(array.offset, 0);
        assert_eq!(array.n_buffers, 2);
        assert_eq!(array.n_children, 0);
        assert!(array.children.is_null());
        assert!(array.dictionary.is_null());
        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        assert!(buffers[0].is_null());
        assert_eq!(buffers[1], data.cast());
        // SAFETY: The value buffer points to the byte held by the export.
        assert_eq!(unsafe { *buffers[1].cast::<u8>() }, 0b0000_0101);

        // SAFETY: The exported schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(schema.format) }, c"b");
        assert_eq!(schema.flags, 0);
        assert_eq!(schema.n_children, 0);
        assert!(schema.children.is_null());
        assert!(schema.dictionary.is_null());
        assert!(weak.upgrade().is_some());

        drop(array);
        assert!(weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn pins_inline_boolean_bitmap_for_export() {
        let bitmap =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0000_0101], 3, 0).expect("valid bitmap");
        let array: Array<bool, ArrayBuffer<1>> = Array::from_buffer(Boolean::from_buffer(bitmap));

        let (array, _schema) = array.export().expect("export array");

        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        // SAFETY: The value buffer points to the inline byte pinned in the
        // export's private allocation.
        assert_eq!(unsafe { *buffers[1].cast::<u8>() }, 0b0000_0101);
    }

    #[test]
    fn rejects_non_zero_boolean_offset() {
        let bitmap =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0001_0100], 3, 2).expect("valid bitmap");
        let array: Array<bool, ArrayBuffer<1>> = Array::from_buffer(Boolean::from_buffer(bitmap));

        let error = array.export().expect_err("non-zero offset");

        assert_eq!(error, ExportError::NonZeroOffset { offset: 2 });
    }
}
