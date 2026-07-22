//! Export [`Array`] values through the Arrow C Data Interface.

extern crate alloc;

use alloc::{boxed::Box, ffi::CString, format, vec::Vec};
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
    layout::{
        ArrayItem, boolean::Boolean, fixed_size_list::FixedSizeList,
        fixed_size_primitive::FixedSizePrimitive,
    },
    length::Length,
    nullability::{NonNullable, Nullable},
};

use crate::{ARROW_FLAG_NULLABLE, ArrowArray, ArrowSchema};

/// A type with an [Arrow C Data format string].
///
/// [Arrow C Data format string]: https://arrow.apache.org/docs/format/CDataInterface.html#data-type-description-format-strings
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
/// Only arrays with an offset of zero are currently supported.
pub trait Export {
    /// Consumes `self` and returns an [`ArrowArray`] and [`ArrowSchema`].
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::NonZeroOffset`] when the array has a non-zero
    /// offset.
    fn export(self) -> Result<(ArrowArray, ArrowSchema), ExportError>;
}

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
        let private = ArrayData::<(), Self>::new((), self.buffers(), self.children()?);

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

        // Pin the layout before asking it for pointers into its storage.
        let mut private = Box::new(ArrayData::<Self, Self> {
            buffers: Self::Buffers::default(),
            children: Self::Children::default(),
            child_pointers: Vec::new(),
            owner: self,
        });
        private.buffers = private.owner.buffers();
        private.children = private.owner.children()?;
        private.set_child_pointers();

        // Convert platform-sized layout metadata into the Arrow C ABI fields.
        let length = i64::try_from(private.owner.len()).expect("array length exceeds i64");
        let null_count = private.owner.null_count();
        let dictionary = private.owner.dictionary();
        let array = private.into_array(length, null_count, dictionary);

        Ok((array, Self::schema()))
    }
}

impl<T, Storage> ArrowArrayLayout for FixedSizePrimitive<T, NonNullable, Storage>
where
    T: FixedSize + ArrowType,
    Storage: Buffer,
{
    type Buffers = [*const c_void; 2];
    type Children = [ArrowArray; 0];

    fn schema() -> ArrowSchema {
        ArrowSchema::flat::<T>()
    }

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
    type Children = [ArrowArray; 0];

    fn schema() -> ArrowSchema {
        ArrowSchema::flat::<Option<T>>()
    }

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
    type Children = [ArrowArray; 0];

    fn schema() -> ArrowSchema {
        ArrowSchema::flat::<bool>()
    }

    fn offset(&self) -> usize {
        self.buffer_ref().bit_offset()
    }

    fn buffers(&self) -> Self::Buffers {
        let values: &[u8] = self.buffer_ref().buffer_ref().borrow();
        [ptr::null(), values.as_ptr().cast()]
    }
}

impl<Storage: Buffer> ArrowArrayLayout for Boolean<Nullable, Storage> {
    type Buffers = [*const c_void; 2];
    type Children = [ArrowArray; 0];

    fn schema() -> ArrowSchema {
        ArrowSchema::flat::<Option<bool>>()
    }

    fn null_count(&self) -> i64 {
        i64::try_from(ValidityBitmap::null_count(self.buffer_ref()))
            .expect("null count exceeds i64")
    }

    fn offset(&self) -> usize {
        let validity = self.buffer_ref();
        // Surface either bitmap offset so the exporter rejects unsupported offsets.
        let validity_offset = validity.bitmap_ref().bit_offset();
        if validity_offset != 0 {
            return validity_offset;
        }
        validity.child_ref().bit_offset()
    }

    fn buffers(&self) -> Self::Buffers {
        let validity = self.buffer_ref();
        let validity_values: &[u8] = validity.bitmap_ref().buffer_ref().borrow();
        let values: &[u8] = validity.child_ref().buffer_ref().borrow();
        [validity_values.as_ptr().cast(), values.as_ptr().cast()]
    }
}

impl<T, const N: usize, Storage> ArrowArrayLayout for FixedSizeList<T, N, NonNullable, Storage>
where
    T: ArrayItem,
    Storage: Buffer,
    T::Memory<Storage>: ArrowArrayLayout + 'static,
{
    type Buffers = [*const c_void; 1];
    type Children = [ArrowArray; 1];

    fn schema() -> ArrowSchema {
        ArrowSchema::fixed_size_list::<N>(<T::Memory<Storage>>::schema())
    }

    fn buffers(&self) -> Self::Buffers {
        [ptr::null()]
    }

    fn children(&self) -> Result<Self::Children, ExportError> {
        Ok([self.buffer_ref().child_ref().child_array()?])
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
    /// Creates pinned private data for an exported array.
    #[allow(clippy::unnecessary_box_returns)]
    fn new(owner: Owner, buffers: Layout::Buffers, children: Layout::Children) -> Box<Self> {
        let mut data = Box::new(Self {
            buffers,
            children,
            child_pointers: Vec::new(),
            owner,
        });
        data.set_child_pointers();
        data
    }

    /// Updates child pointers after the child arrays are pinned.
    fn set_child_pointers(&mut self) {
        self.child_pointers = self
            .children
            .as_mut()
            .iter_mut()
            .map(ptr::from_mut)
            .collect();
    }

    /// Builds an [`ArrowArray`] backed by this private data.
    fn into_array(
        mut self: Box<Self>,
        length: i64,
        null_count: i64,
        dictionary: *mut ArrowArray,
    ) -> ArrowArray {
        let n_buffers =
            i64::try_from(self.buffers.as_ref().len()).expect("buffer count exceeds i64");
        let n_children = i64::try_from(self.child_pointers.len()).expect("child count exceeds i64");

        let buffers = self.buffers.as_mut();
        let buffer_pointers = if buffers.is_empty() {
            ptr::null_mut()
        } else {
            buffers.as_mut_ptr()
        };
        let children = if self.child_pointers.is_empty() {
            ptr::null_mut()
        } else {
            self.child_pointers.as_mut_ptr()
        };
        let private_data = Box::into_raw(self).cast();

        ArrowArray {
            length,
            null_count,
            offset: 0,
            n_buffers,
            n_children,
            buffers: buffer_pointers,
            children,
            dictionary,
            release: Some(release_array::<Self>),
            private_data,
        }
    }
}

impl<T, Storage> Export for Array<T, Storage>
where
    T: ArrayItem,
    Storage: Buffer,
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

    /// Builds a fixed-size-list schema and retains its child schema.
    fn fixed_size_list<const N: usize>(child: Self) -> Self {
        let format_string = CString::new(format!("+w:{N}")).expect("valid fixed-size list format");
        let mut private = Box::new(FixedSizeListSchemaData {
            format: format_string,
            children: [child],
            child_pointers: [ptr::null_mut()],
        });
        private.child_pointers[0] = ptr::from_mut(&mut private.children[0]);

        let format_pointer = private.format.as_ptr();
        let children = private.child_pointers.as_mut_ptr();
        let private_data = Box::into_raw(private).cast();
        Self {
            format: format_pointer,
            name: c"".as_ptr(),
            metadata: ptr::null(),
            flags: 0,
            n_children: 1,
            children,
            dictionary: ptr::null_mut(),
            release: Some(release_schema::<FixedSizeListSchemaData>),
            private_data,
        }
    }
}

/// Data retained by `ArrowSchema::private_data` for a fixed-size list.
struct FixedSizeListSchemaData {
    /// Arrow C Data type format.
    format: CString,
    /// Child schemas owned by the export.
    children: [ArrowSchema; 1],
    /// Arrow C Data child pointers.
    child_pointers: [*mut ArrowSchema; 1],
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
        collection::flatten::Flatten,
        layout::{
            boolean::Boolean, fixed_size_list::FixedSizeList,
            fixed_size_primitive::FixedSizePrimitive,
        },
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
        let narrow_array: Array<i32, ArcBuffer> =
            Array::from_buffer(FixedSizePrimitive::from_buffer(values));

        let (array, schema) = narrow_array.export().expect("export array");

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
        assert_eq!(
            // SAFETY: The second buffer points to three i32 values held by the export.
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
        let narrow_array: Array<i32, ArrayBuffer<3>> =
            Array::from_buffer(FixedSizePrimitive::from_buffer([1, 2, 3]));

        let (array, _schema) = narrow_array.export().expect("export array");

        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        assert_eq!(
            // SAFETY: The second buffer points to three inline i32 values pinned in
            // the export's private allocation.
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
        let narrow_array: Array<Option<i32>, ArcBuffer> =
            Array::from_buffer(FixedSizePrimitive::from_buffer(validity));

        let (array, schema) = narrow_array.export().expect("export array");

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
        assert_eq!(
            // SAFETY: The value buffer points to three i32 values.
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
        let narrow_array: Array<bool, ArcBuffer> = Array::from_buffer(Boolean::from_buffer(bitmap));

        let (array, schema) = narrow_array.export().expect("export array");

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
    fn exports_nullable_boolean_bitmaps_without_copying() {
        let value_bytes = Arc::<[u8]>::from([0b0000_0001]);
        let validity_values = Arc::<[u8]>::from([0b0000_0101]);
        let values_weak = Arc::downgrade(&value_bytes);
        let validity_weak = Arc::downgrade(&validity_values);
        let values_data = value_bytes.as_ptr();
        let validity_data = validity_values.as_ptr();
        let value_bitmap =
            Bitmap::<ArcBuffer>::try_from_parts(value_bytes, 3, 0).expect("valid bitmap");
        let validity_bitmap =
            Bitmap::<ArcBuffer>::try_from_parts(validity_values, 3, 0).expect("valid bitmap");
        let validity =
            Validity::try_from_parts(value_bitmap, validity_bitmap).expect("valid parts");
        let narrow_array: Array<Option<bool>, ArcBuffer> =
            Array::from_buffer(Boolean::from_buffer(validity));

        let (array, schema) = narrow_array.export().expect("export array");

        assert_eq!(array.length, 3);
        assert_eq!(array.null_count, 1);
        assert_eq!(array.offset, 0);
        assert_eq!(array.n_buffers, 2);
        assert_eq!(array.n_children, 0);
        // SAFETY: The exported array owns a two-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 2) };
        assert_eq!(buffers[0], validity_data.cast());
        assert_eq!(buffers[1], values_data.cast());
        // SAFETY: Both buffers point to bytes retained by the export.
        assert_eq!(unsafe { *buffers[0].cast::<u8>() }, 0b0000_0101);
        // SAFETY: The value buffer points to a byte retained by the export.
        assert_eq!(unsafe { *buffers[1].cast::<u8>() }, 0b0000_0001);

        // SAFETY: The exported schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(schema.format) }, c"b");
        assert_eq!(schema.flags, ARROW_FLAG_NULLABLE);
        assert!(values_weak.upgrade().is_some());
        assert!(validity_weak.upgrade().is_some());

        drop(array);
        assert!(values_weak.upgrade().is_none());
        assert!(validity_weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn rejects_non_zero_nullable_boolean_offsets() {
        let offset_values =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0001_0100], 3, 2).expect("valid bitmap");
        let zero_validity =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0000_0111], 3, 0).expect("valid bitmap");
        let offset_values_validity =
            Validity::try_from_parts(offset_values, zero_validity).expect("valid parts");
        let offset_values_array: Array<Option<bool>, ArrayBuffer<1>> =
            Array::from_buffer(Boolean::from_buffer(offset_values_validity));
        assert_eq!(
            offset_values_array
                .export()
                .expect_err("non-zero value offset"),
            ExportError::NonZeroOffset { offset: 2 }
        );

        let zero_values =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0000_0101], 3, 0).expect("valid bitmap");
        let offset_validity =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0001_1100], 3, 2).expect("valid bitmap");
        let offset_validity_values =
            Validity::try_from_parts(zero_values, offset_validity).expect("valid parts");
        let offset_validity_array: Array<Option<bool>, ArrayBuffer<1>> =
            Array::from_buffer(Boolean::from_buffer(offset_validity_values));
        assert_eq!(
            offset_validity_array
                .export()
                .expect_err("non-zero validity offset"),
            ExportError::NonZeroOffset { offset: 2 }
        );
    }

    #[test]
    fn exports_fixed_size_list_child_without_copying_values() {
        let value_storage = Arc::<[i32]>::from([1, 2, 3, 4]);
        let weak = Arc::downgrade(&value_storage);
        let data = value_storage.as_ptr();
        let values_layout = FixedSizePrimitive::from_buffer(value_storage);
        let flattened = Flatten::try_from_parts(values_layout).expect("valid fixed-size list");
        let narrow_array: Array<[i32; 2], ArcBuffer> =
            Array::from_buffer(FixedSizeList::from_buffer(flattened));

        let (array, schema) = narrow_array.export().expect("export array");

        assert_eq!(array.length, 2);
        assert_eq!(array.null_count, 0);
        assert_eq!(array.offset, 0);
        assert_eq!(array.n_buffers, 1);
        assert_eq!(array.n_children, 1);
        // SAFETY: The exported array owns a one-entry buffer pointer array.
        let buffers = unsafe { slice::from_raw_parts(array.buffers, 1) };
        assert!(buffers[0].is_null());
        // SAFETY: The exported array owns a one-entry child pointer array.
        let array_children = unsafe { slice::from_raw_parts(array.children, 1) };
        // SAFETY: The child pointer refers to an array retained by the parent.
        let child_array = unsafe { &*array_children[0] };
        assert_eq!(child_array.length, 4);
        assert_eq!(child_array.n_buffers, 2);
        assert_eq!(child_array.n_children, 0);
        // SAFETY: The child owns a two-entry buffer pointer array.
        let child_buffers = unsafe { slice::from_raw_parts(child_array.buffers, 2) };
        assert!(child_buffers[0].is_null());
        assert_eq!(child_buffers[1], data.cast());
        assert_eq!(
            // SAFETY: The value buffer points to four values retained by the parent.
            unsafe { slice::from_raw_parts(child_buffers[1].cast::<i32>(), 4) },
            [1, 2, 3, 4]
        );

        // SAFETY: The exported schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(schema.format) }, c"+w:2");
        assert_eq!(schema.flags, 0);
        assert_eq!(schema.n_children, 1);
        // SAFETY: The schema owns a one-entry child pointer array.
        let schema_children = unsafe { slice::from_raw_parts(schema.children, 1) };
        // SAFETY: The child pointer refers to a schema retained by the parent.
        let child_schema = unsafe { &*schema_children[0] };
        // SAFETY: The child schema has a live, null-terminated format string.
        assert_eq!(unsafe { CStr::from_ptr(child_schema.format) }, c"i");
        assert_eq!(child_schema.flags, 0);
        assert_eq!(child_schema.n_children, 0);
        assert!(weak.upgrade().is_some());

        drop(array);
        assert!(weak.upgrade().is_none());
        drop(schema);
    }

    #[test]
    fn rejects_non_zero_fixed_size_list_child_offset() {
        let bitmap = Bitmap::<ArrayBuffer<4>>::try_from_parts([0b0011_1100, 0, 0, 0], 4, 2)
            .expect("valid bitmap");
        let validity = Validity::try_from_parts([1, 2, 3, 4], bitmap).expect("valid parts");
        let values_layout = FixedSizePrimitive::from_buffer(validity);
        let flattened = Flatten::try_from_parts(values_layout).expect("valid fixed-size list");
        let array: Array<[Option<i32>; 2], ArrayBuffer<4>> =
            Array::from_buffer(FixedSizeList::from_buffer(flattened));

        let error = array.export().expect_err("non-zero child offset");

        assert_eq!(error, ExportError::NonZeroOffset { offset: 2 });
    }

    #[test]
    fn pins_inline_boolean_bitmap_for_export() {
        let bitmap =
            Bitmap::<ArrayBuffer<1>>::try_from_parts([0b0000_0101], 3, 0).expect("valid bitmap");
        let narrow_array: Array<bool, ArrayBuffer<1>> =
            Array::from_buffer(Boolean::from_buffer(bitmap));

        let (array, _schema) = narrow_array.export().expect("export array");

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
