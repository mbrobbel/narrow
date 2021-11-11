use crate::{Length, Primitive};
use std::{
    alloc::{self, Layout},
    any,
    fmt::{Debug, Formatter, Result},
    hash::{Hash, Hasher},
    iter::Copied,
    mem,
    ops::Deref,
    ptr::{self, NonNull},
    slice::{self, Iter},
};

// todo(mb): replace with allocator api (https://github.com/rust-lang/rust/issues/32838)
// todo(mb): add defaults for alignment const generic (https://github.com/rust-lang/rust/issues/44580)

/// Default exponent of power-of-two alignment for buffers. (64 bytes)
pub(crate) const ALIGN: usize = 6;

/// A contiguous immutable memory buffer for data.
///
/// Generic over the element type `T` stored in this buffer and the power-of-two
/// alignment `A` of the buffer.
///
/// - `T` must implement [Copy]. The elements of the buffer [can't have
///   destructors](https://doc.rust-lang.org/std/ops/trait.Drop.html#copy-and-drop-are-exclusive).
/// - `A` is the exponent of a power-of-two alignment.
///
/// An important invariant of a [Buffer] is its memory [Layout].
/// - The layout's size is always the length (invariant because buffer is
///   immutable) multiplied with the element size, with trailing padded added to
///   round up to a multiple of the alignment.
/// - The layout's alignment is `1 << A`.
///
/// This is currently implemented using low-level unsafe code. When the
/// [Allocator](std::alloc::Allocator) trait is stabilized a wrapper around a
/// [Vec] with a custom allocator implementation (for the alignment and padding
/// requirements) can replace most of the code here.
// todo(mb): const A: NonZeroUsize
pub struct Buffer<T, const A: usize> {
    /// The pointer to the memory location of the buffer.
    ptr: NonNull<T>,
    /// The length of this buffer i.e. the number of elements.
    len: usize,
}

impl<T, const A: usize> Buffer<T, A>
where
    T: Primitive, // This bound for all methods that can construct Buffers
{
    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr() as *const T
    }

    pub fn as_slice(&self) -> &[T] {
        self
    }

    pub fn get(&self, index: usize) -> Option<T> {
        self.deref().get(index).copied()
    }

    pub fn iter(&self) -> Copied<Iter<'_, T>> {
        self.into_iter()
    }

    /// Returns the number of bytes in this buffer.
    pub fn size(&self) -> usize {
        // The number of bytes allocated by a buffer is not the number of
        // elements multiplied by the size of each element. It includes the
        // additional trailing bytes that were allocated to make the allocation
        // size a multiple of the alignment.
        self.layout().size()
    }

    /// Returns the alignment (in bytes) of this buffer.
    pub fn align(&self) -> usize {
        // Power-of-two alignment of the buffer is set by the const generic A on
        // the buffer.
        1 << A
    }

    /// Returns the number of padding bytes in this memory block.
    pub fn padding(&self) -> usize {
        // The number of padding bits in a buffer are the size of allocation
        // subtracted by the part used by the current number of elements in the
        // buffer.
        self.size() - self.len * mem::size_of::<T>()
    }

    /// Returns an new Buffer.
    /// Safety:
    /// - ptr must be non-null
    /// - ptr must be allocated using the Layout for the given layout and
    ///   alignment.
    /// - len must be non-zero
    pub(crate) unsafe fn new_unchecked(ptr: *mut T, len: usize) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr),
            len,
        }
    }

    /// Constructs a [Buffer] from a [slice].
    fn from_slice(slice: &[T]) -> Self {
        // Allocate buffer that holds `N` elements.
        let ptr = unsafe { Self::alloc(slice.len()) };

        // Copy the elements from the array into the new buffer.
        // Safety
        // - Conditions to prevent undefined behavior are met:
        //  - source is assumed to have its invariants maintained.
        //  - Checked allocation for destination above.
        //  - source is assumed to be properly aligned.
        //  - Regions don't overlap because destination was allocated above.
        unsafe { ptr::copy_nonoverlapping(slice.as_ptr(), ptr, slice.len()) }

        Self {
            ptr:
            // Safety:
            // - Pointer is non-null as this is checked in the `alloc`
            //   function.
            unsafe { NonNull::new_unchecked(ptr) },
            len: slice.len(),
        }
    }

    /// Allocates the memory for a [Buffer] with alignment `A` and provided length
    /// (number of elements).
    ///
    /// This method will likely be deprecated when Allocator APIs are stabilized.
    pub(crate) unsafe fn alloc(len: usize) -> *mut T {
        // Safety
        // - Size condition is checked in `layout_len` function.
        // - Alignment condition is checked in `layout_len` function.
        let ptr = alloc::alloc(Self::layout_len(len)) as *mut T;

        // Make sure the allocation did not fail.
        assert!(!ptr.is_null(), "Allocation failed");

        // Return the pointer.
        ptr
    }

    /// Attempt to reallocate the memory for a [Buffer] so that it can hold the
    /// new length (number of elements).
    ///
    /// When the layout for the new length matches the layout of the old length,
    /// this does not allocate and simply returns the given ptr, because the
    /// padding from the previous allocation can hold the required new length.
    ///
    /// When the current allocation can't hold the new length, a new allocation
    /// is attempted with the new layout. The values from the previous
    /// allocation are copied from the source, and the source location is
    /// deallocated.
    ///
    /// # Safety
    /// This method is unsafe and its behavior is undefined unless the following
    /// conditions are met:
    /// - ptr must be non-null and allocated with layout based on current length
    /// - ptr can't be used after invoking this function because it might be
    ///   deallocated
    pub(crate) unsafe fn realloc(ptr: *mut T, current_len: usize, new_len: usize) -> *mut T {
        let current_layout = Self::layout_len(current_len);
        let new_layout = Self::layout_len(new_len);

        // Check if the current allocation layout can hold the new length.
        if current_layout == new_layout {
            // No need to reallocate. There is enough capacity in the padding to
            // store `new_len` elements.
            ptr
        } else {
            // Allocate new buffer and copy contents from source.
            let new_ptr = Self::alloc(new_len);
            ptr::copy_nonoverlapping(ptr as *const T, new_ptr, current_len);

            // Deallocate previous allocation.
            alloc::dealloc(ptr as *mut u8, current_layout);

            // Return the new pointer.
            new_ptr
        }
    }
}

impl<T, const A: usize> Buffer<T, A> {
    /// Returns the [Layout] for a [Buffer] with alignment `A` and provided length
    /// (number of elements `T`).
    pub(crate) fn layout_len(len: usize) -> Layout {
        assert!(len != 0, "Zero-sized layouts are not supported");

        // Power-of-two alignment.
        let align = 1 << A;

        // todo(mb): replace const generic with NonZeroUsize
        assert!(align != 0, "Align can't be zero");

        // Make sure the alignment is correct.
        assert!(
            align % mem::align_of::<T>() == 0,
            "Alignment `A` must be a multiple of the ABI-required minimum alignment of type `T`"
        );

        // No additional padding between elements.
        let size = len * mem::size_of::<T>();

        // Construct the Layout based on size and align and pad to multiple of alignment.
        Layout::from_size_align(size, align)
            .map(|layout| layout.pad_to_align())
            .expect("Allocation size overflow")
    }

    /// Returns the [Layout] of the [Buffer].
    fn layout(&self) -> Layout {
        let size = self.len * mem::size_of::<T>();
        let align = 1 << A;
        // Safety:
        // - This can only be called if this Buffer was successfully constructed.
        unsafe { Layout::from_size_align_unchecked(size, align).pad_to_align() }
    }
}

impl<T, const A: usize> AsRef<[u8]> for Buffer<T, A> {
    fn as_ref(&self) -> &[u8] {
        // Safety:
        // - Length (number of elements) is an invariant of an immutable buffer.
        unsafe {
            slice::from_raw_parts(
                self.ptr.as_ptr() as *const u8,
                self.len * mem::size_of::<T>(),
            )
        }
    }
}

impl<T, const A: usize> Clone for Buffer<T, A>
where
    T: Primitive,
{
    fn clone(&self) -> Self {
        Self::from_slice(self)
    }
}

impl<T, const A: usize> Debug for Buffer<T, A>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct(&format!("Buffer<{}, {}>", any::type_name::<T>(), A))
            .field("values", &self.deref())
            .finish()
    }
}

impl<T, const A: usize> Default for Buffer<T, A>
where
    T: Primitive,
{
    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
        }
    }
}

impl<T, const A: usize> Deref for Buffer<T, A> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // Safety:
        // - Conditions that would result in undefined behavior are met by the
        //   invariants of the buffer (layout, allocation and length).
        unsafe { slice::from_raw_parts(self.ptr.as_ptr() as *const T, self.len) }
    }
}

impl<T, const A: usize> Drop for Buffer<T, A> {
    fn drop(&mut self) {
        // Don't attempt to deallocate empty buffers.
        if self.len != 0 {
            // Manually deallocate the memory buffer.
            // Safety
            // - The ptr was allocated using the same default allocator and
            //   non-null as checked above.
            // - The layout is invariant because the length, alignment and padding
            //   are invariant.
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, self.layout());
            }
        }
    }
}

impl<T, const A: usize> Eq for Buffer<T, A> where for<'a> &'a [T]: PartialEq {}

impl<T, const A: usize> FromIterator<T> for Buffer<T, A>
where
    T: Primitive,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut iter = iter.into_iter();

        match iter.next() {
            Some(value) => {
                // Allocate some memory based on the size hint.
                let (lower_bound, _) = iter.size_hint();
                let mut ptr = unsafe { Self::alloc(lower_bound + 1) };

                // Write first value.
                unsafe { ptr.write(value) };

                // Write at least `len` more values to the buffer.
                let mut len = 1;
                while len < lower_bound {
                    unsafe {
                        ptr.add(len).write(
                            iter.next()
                                .expect("reported lower bound of size hint incorrect"),
                        );
                    }
                    len += 1;
                }

                // Add the remaining items, while making sure the allocated
                // layout can hold the number of elements.
                for value in iter {
                    ptr = unsafe { Self::realloc(ptr, len, len + 1) };
                    unsafe { ptr.add(len).write(value) };
                    len += 1;
                }

                Self {
                    ptr: unsafe { NonNull::new_unchecked(ptr) },
                    len,
                }
            }
            None => Self::default(),
        }
    }
}

// todo(mb): test
impl<T, const A: usize> Hash for Buffer<T, A>
where
    T: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        Hash::hash_slice(&self[..], state);
        self.len.hash(state);
    }
}

impl<'a, T, const A: usize> IntoIterator for &'a Buffer<T, A>
where
    T: Copy,
{
    type Item = T;
    type IntoIter = Copied<Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter().copied()
    }
}

impl<T, const A: usize> Length for Buffer<T, A> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<T, const A: usize> PartialEq for Buffer<T, A>
where
    for<'a> &'a [T]: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && (self.len == 0 || &self[..] == &other[..])
    }
}

/// Buffer is [Send] because the buffer is immutable.
unsafe impl<T, const A: usize> Send for Buffer<T, A> {}

/// Buffer is [Sync] because the buffer is immutable.
unsafe impl<T, const A: usize> Sync for Buffer<T, A> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Zero-sized layouts are not supported")]
    fn layout_zero_sized() {
        Buffer::<u8, 0>::layout_len(0);
    }

    #[test]
    #[should_panic(expected = "Allocation size overflow")]
    fn layout_overflow() {
        Buffer::<u8, 6>::layout_len(usize::MAX - 62);
    }

    #[test]
    #[should_panic(
        expected = "Alignment `A` must be a multiple of the ABI-required minimum alignment of type `T`"
    )]
    fn layout_bad_align() {
        // Should fail because align_of::<u64>() is 8 (1 << 3).
        Buffer::<u64, 2>::layout_len(1234);
    }

    #[test]
    fn layout_size() {
        assert_eq!(Buffer::<u8, 0>::layout_len(1).size(), 1);
        assert_eq!(Buffer::<u8, 5>::layout_len(1).size(), 32);
        assert_eq!(Buffer::<u8, 5>::layout_len(32).size(), 32);
        assert_eq!(Buffer::<u8, 5>::layout_len(33).size(), 64);
        assert_eq!(Buffer::<u8, 6>::layout_len(1).size(), 64);
        assert_eq!(Buffer::<u8, 6>::layout_len(64).size(), 64);
        assert_eq!(Buffer::<u8, 6>::layout_len(65).size(), 128);
        assert_eq!(Buffer::<u32, 6>::layout_len(5).size(), 64);
        assert_eq!(Buffer::<f64, 6>::layout_len(8).size(), 64);
        assert_eq!(Buffer::<f64, 6>::layout_len(9).size(), 128);
    }

    #[test]
    fn memory() {
        let buffer = [1, 2, 3, 4].into_iter().collect::<Buffer<u8, 6>>();
        assert_eq!(buffer.len(), 4);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.size(), 64);
        assert_eq!(buffer.align(), 64);
        assert_eq!(buffer.padding(), 60);
    }

    #[test]
    fn as_ref_u8() {
        let vec = vec![42u32, u32::MAX, 0xc0fefe];
        let buffer: Buffer<_, 7> = vec.into_iter().collect();
        assert_eq!(
            AsRef::<[u8]>::as_ref(&buffer),
            &[42u8, 0, 0, 0, 255, 255, 255, 255, 254, 254, 192, 0]
        );
    }

    #[test]
    fn deref() {
        let buffer: Buffer<_, 3> = [1u32, 2, 3, 4].into_iter().collect();
        assert_eq!(buffer.len(), 4);
        assert_eq!(&buffer[2..], &[3, 4]);
    }

    #[test]
    fn default() {
        let buffer: Buffer<u8, 6> = Buffer::default();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert!(buffer.first().is_none());
        assert!(buffer.get(1234).is_none());
        assert!(buffer.iter().next().is_none());
        let slice = &buffer[..];
        let empty_slice: &[u8] = &[];
        assert_eq!(slice, empty_slice);
        let bytes: &[u8] = buffer.as_ref();
        assert_eq!(bytes, empty_slice);
    }

    #[test]
    fn from_iter() {
        let vec = vec![1u32, 2, 3, 4];
        let buffer = vec.clone().into_iter().collect::<Buffer<_, 6>>();
        assert_eq!(buffer.len(), 4);
        assert_eq!(&vec[..], &buffer[..]);
    }

    #[test]
    fn from_iter_ref() {
        let vec = vec![1u32, 2, 3, 4];
        let buffer = vec.iter().copied().collect::<Buffer<_, 4>>();
        assert_eq!(buffer.len(), 4);
        assert_eq!(&vec[..], &buffer[..]);
    }

    #[test]
    fn into_iter() {
        let vec = vec![1u32, 2, 3, 4];
        let other = vec
            .iter()
            .copied()
            .collect::<Buffer<_, 5>>()
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(vec, other);
    }
}
