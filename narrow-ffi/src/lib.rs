//! Arrow C Data Interface support for Narrow.
//!
//! See [The Arrow C data interface] specification.
//!
//! [The Arrow C data interface]: https://arrow.apache.org/docs/format/CDataInterface.html

#![no_std]
#![deny(missing_docs, unsafe_op_in_unsafe_fn)]

use core::{
    ffi::{c_char, c_void},
    ptr,
};

mod export;
pub use export::{ArrowPrimitive, Export};

/// Dictionary values are ordered.
pub const ARROW_FLAG_DICTIONARY_ORDERED: i64 = 1;

/// The field is nullable.
pub const ARROW_FLAG_NULLABLE: i64 = 2;

/// Map keys are sorted.
pub const ARROW_FLAG_MAP_KEYS_SORTED: i64 = 4;

/// The Arrow C Data Interface schema structure.
#[repr(C)]
#[derive(Debug)]
pub struct ArrowSchema {
    /// Mandatory null-terminated UTF-8 type format string.
    format: *const c_char,
    /// Optional null-terminated UTF-8 field name.
    name: *const c_char,
    /// Optional binary-encoded metadata, which is not null-terminated.
    metadata: *const c_char,
    /// Bitfield of the `ARROW_FLAG_*` constants.
    flags: i64,
    /// Number of child schemas.
    n_children: i64,
    /// C array of `n_children` child schema pointers.
    children: *mut *mut Self,
    /// Dictionary value schema, or null when the type is not dictionary encoded.
    dictionary: *mut Self,
    /// Producer callback that releases the schema; [`None`] marks it released.
    release: Option<unsafe extern "C" fn(*mut Self)>,
    /// Optional opaque producer-owned data used by the release callback.
    private_data: *mut c_void,
}

impl ArrowSchema {
    /// Returns whether this schema has been released.
    #[must_use]
    pub const fn is_released(&self) -> bool {
        self.release.is_none()
    }
}

impl Default for ArrowSchema {
    fn default() -> Self {
        Self {
            format: ptr::null(),
            name: ptr::null(),
            metadata: ptr::null(),
            flags: 0,
            n_children: 0,
            children: ptr::null_mut(),
            dictionary: ptr::null_mut(),
            release: None,
            private_data: ptr::null_mut(),
        }
    }
}

impl Drop for ArrowSchema {
    fn drop(&mut self) {
        if let Some(release) = self.release {
            // SAFETY: A live Arrow C Data structure owns a producer-provided
            // callback that accepts the address of the structure being released.
            unsafe { release(self) };
        }
    }
}

/// The Arrow C Data Interface array structure.
#[repr(C)]
#[derive(Debug)]
pub struct ArrowArray {
    /// Logical number of elements in the array.
    length: i64,
    /// Number of null elements, or `-1` when unknown.
    null_count: i64,
    /// Non-negative logical element offset into the physical buffers.
    offset: i64,
    /// Number of physical buffers, excluding child buffers.
    n_buffers: i64,
    /// Number of child arrays.
    n_children: i64,
    /// C array of `n_buffers` physical buffer pointers.
    buffers: *mut *const c_void,
    /// C array of `n_children` child array pointers.
    children: *mut *mut Self,
    /// Dictionary values, or null when the array is not dictionary encoded.
    dictionary: *mut Self,
    /// Producer callback that releases the array; [`None`] marks it released.
    release: Option<unsafe extern "C" fn(*mut Self)>,
    /// Optional opaque producer-owned data used by the release callback.
    private_data: *mut c_void,
}

impl ArrowArray {
    /// Returns whether this array has been released.
    #[must_use]
    pub const fn is_released(&self) -> bool {
        self.release.is_none()
    }
}

impl Default for ArrowArray {
    fn default() -> Self {
        Self {
            length: 0,
            null_count: 0,
            offset: 0,
            n_buffers: 0,
            n_children: 0,
            buffers: ptr::null_mut(),
            children: ptr::null_mut(),
            dictionary: ptr::null_mut(),
            release: None,
            private_data: ptr::null_mut(),
        }
    }
}

impl Drop for ArrowArray {
    fn drop(&mut self) {
        if let Some(release) = self.release {
            // SAFETY: A live Arrow C Data structure owns a producer-provided
            // callback that accepts the address of the structure being released.
            unsafe { release(self) };
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_pointer_width = "64")]
    use core::mem::{align_of, offset_of, size_of};
    use core::sync::atomic::{AtomicUsize, Ordering};

    use super::{ArrowArray, ArrowSchema};

    static ARRAY_RELEASES: AtomicUsize = AtomicUsize::new(0);
    static SCHEMA_RELEASES: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn release_array(array: *mut ArrowArray) {
        ARRAY_RELEASES.fetch_add(1, Ordering::Relaxed);
        // SAFETY: The callback receives the live structure from its `Drop` impl.
        unsafe { (*array).release = None };
    }

    unsafe extern "C" fn release_schema(schema: *mut ArrowSchema) {
        SCHEMA_RELEASES.fetch_add(1, Ordering::Relaxed);
        // SAFETY: The callback receives the live structure from its `Drop` impl.
        unsafe { (*schema).release = None };
    }

    #[test]
    fn empty_structures_are_released() {
        assert!(ArrowArray::default().is_released());
        assert!(ArrowSchema::default().is_released());
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn structures_match_the_64_bit_c_abi() {
        assert_eq!(align_of::<ArrowSchema>(), 8);
        assert_eq!(size_of::<ArrowSchema>(), 72);
        assert_eq!(offset_of!(ArrowSchema, format), 0);
        assert_eq!(offset_of!(ArrowSchema, name), 8);
        assert_eq!(offset_of!(ArrowSchema, metadata), 16);
        assert_eq!(offset_of!(ArrowSchema, flags), 24);
        assert_eq!(offset_of!(ArrowSchema, n_children), 32);
        assert_eq!(offset_of!(ArrowSchema, children), 40);
        assert_eq!(offset_of!(ArrowSchema, dictionary), 48);
        assert_eq!(offset_of!(ArrowSchema, release), 56);
        assert_eq!(offset_of!(ArrowSchema, private_data), 64);

        assert_eq!(align_of::<ArrowArray>(), 8);
        assert_eq!(size_of::<ArrowArray>(), 80);
        assert_eq!(offset_of!(ArrowArray, length), 0);
        assert_eq!(offset_of!(ArrowArray, null_count), 8);
        assert_eq!(offset_of!(ArrowArray, offset), 16);
        assert_eq!(offset_of!(ArrowArray, n_buffers), 24);
        assert_eq!(offset_of!(ArrowArray, n_children), 32);
        assert_eq!(offset_of!(ArrowArray, buffers), 40);
        assert_eq!(offset_of!(ArrowArray, children), 48);
        assert_eq!(offset_of!(ArrowArray, dictionary), 56);
        assert_eq!(offset_of!(ArrowArray, release), 64);
        assert_eq!(offset_of!(ArrowArray, private_data), 72);
    }

    #[test]
    fn drop_calls_release_once() {
        ARRAY_RELEASES.store(0, Ordering::Relaxed);
        SCHEMA_RELEASES.store(0, Ordering::Relaxed);

        let array = ArrowArray {
            release: Some(release_array),
            ..ArrowArray::default()
        };
        let schema = ArrowSchema {
            release: Some(release_schema),
            ..ArrowSchema::default()
        };

        drop(array);
        drop(schema);

        assert_eq!(ARRAY_RELEASES.load(Ordering::Relaxed), 1);
        assert_eq!(SCHEMA_RELEASES.load(Ordering::Relaxed), 1);
    }
}
