use crate::{Data, Primitive};
use std::{
    alloc::{self, Layout},
    any,
    borrow::Borrow,
    fmt::{Debug, Formatter, Result},
    iter::{Copied, FromIterator},
    mem,
    ops::Deref,
    ptr::{self, NonNull},
    slice::{self, Iter},
};

// todo(mb): replace with allocator api (https://github.com/rust-lang/rust/issues/32838)
// todo(mb): add defaults for alignment const generic (https://github.com/rust-lang/rust/issues/44580)
// todo(mb): implement partial eq and eq for buffer (layout aware?)
// todo(mb): implement clone
// todo(mb): implement hash

/// A contiguous immutable memory buffer for Arrow data.
///
/// Generic over the element type `T` stored in this buffer and the alignment
/// `A` of the buffer.
///
/// - `T` must implement [Copy]. The elements of the buffer
/// [can't have destructors](https://doc.rust-lang.org/std/ops/trait.Drop.html#copy-and-drop-are-exclusive).
/// - `A` is the exponent of a power-of-two alignment.
///
/// An important invariant of a [Buffer] is its memory [Layout].
/// - The layout's size is always the length (invariant because buffer is
///   immutable) multiplied with the element size.
/// - The layout's alignment is `A`.
/// - The layout is based on the size and alignment as described above, with
///   trailing padding to round up to the alignment.
/// Because of this invariant this struct stores just a [ptr] and a length, and
/// can be used as a slice via a [Deref] implementation.
///
/// This is currently implemented using low-level unsafe code. When the
/// [Allocator](std::alloc::Allocator) trait is stabilized a wrapper around a
/// [Vec] with a custom allocator implementation (for the alignment and padding
/// requirements) can replace all the code here.
pub struct Buffer<T, const A: usize>
where
    T: Copy, // This is not [Primitive] because [Buffer] is also used in
             // [Bitmap] that stores [usize] elements.
{
    /// The pointer to the memory location of the buffer.
    ptr: NonNull<T>,
    /// The length of this buffer i.e. the number of elements.
    len: usize,
}

/// Buffer is [Send] because the buffer is immutable.
unsafe impl<T, const A: usize> Send for Buffer<T, A> where T: Copy {}

/// Buffer is [Sync] because the buffer is immutable.
unsafe impl<T, const A: usize> Sync for Buffer<T, A> where T: Copy {}

impl<T, const A: usize> Buffer<T, A>
where
    T: Copy,
{
    /// Returns an empty [Buffer].
    ///
    /// Because buffers are immutable the [Buffer] will always be empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::Buffer;
    ///
    /// let empty = Buffer::<u8, 0>::empty();
    ///
    /// assert!(empty.is_empty());
    /// assert_eq!(&empty[..], &[]);
    /// ```
    pub fn empty() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
        }
    }

    /// Extracts a slice containing the entire buffer.
    ///
    /// Equivalent to `&buffer[..]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::Buffer;
    ///
    /// let buffer: Buffer<_, 3> = [1u8, 2, 3, 4].into();
    ///
    /// assert_eq!(buffer.as_slice(), &buffer[..]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        &self
    }

    /// Returns an new Buffer.
    /// todo(mb): document safety issues
    pub(crate) unsafe fn new_unchecked(ptr: *mut T, len: usize) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr),
            len,
        }
    }

    /// Returns the [Layout] of the [Buffer].
    fn layout(&self) -> Layout {
        layout::<T, A>(self.len())
    }

    /// Constructs a [Buffer] from a [slice].
    fn from_slice(slice: &[T]) -> Self {
        // Allocate buffer that holds `N` elements.
        let ptr = unsafe { alloc::<T, A>(layout::<T, A>(slice.len())) };

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
}

impl<T, const A: usize> Debug for Buffer<T, A>
where
    T: Copy + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct(&format!("Buffer<{}, {}>", any::type_name::<T>(), A))
            .field("values", &self.deref())
            .finish()
    }
}

impl<T, const A: usize> Default for Buffer<T, A>
where
    T: Copy,
{
    fn default() -> Self {
        Buffer::empty()
    }
}

impl<T, const A: usize> Drop for Buffer<T, A>
where
    T: Copy,
{
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

impl<T, const A: usize> Data for Buffer<T, A>
where
    T: Copy,
{
    fn len(&self) -> usize {
        self.len
    }

    fn null_count(&self) -> usize {
        0
    }

    fn valid_count(&self) -> usize {
        self.len
    }
}

impl<T, const A: usize> AsRef<Buffer<T, A>> for Buffer<T, A>
where
    T: Copy,
{
    fn as_ref(&self) -> &Buffer<T, A> {
        self
    }
}

impl<T, const A: usize> AsRef<[u8]> for Buffer<T, A>
where
    T: Copy,
{
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

impl<T, const A: usize> Borrow<[T]> for Buffer<T, A>
where
    T: Copy,
{
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T, const A: usize> Deref for Buffer<T, A>
where
    T: Copy,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // Safety:
        // - Conditions that would result in undefined behavior are met by the
        //   invariants of the buffer (layout, allocation and length).
        unsafe { slice::from_raw_parts(self.ptr.as_ptr() as *const T, self.len) }
    }
}

impl<T, const N: usize, const A: usize> From<[T; N]> for Buffer<T, A>
where
    T: Primitive,
{
    fn from(array: [T; N]) -> Self {
        Self::from_slice(&array)
    }
}

impl<T, const A: usize> From<Box<[T]>> for Buffer<T, A>
where
    T: Primitive,
{
    fn from(boxed_slice: Box<[T]>) -> Self {
        Self::from_slice(&boxed_slice)
    }
}

impl<T, const A: usize> From<&[T]> for Buffer<T, A>
where
    T: Primitive,
{
    fn from(slice: &[T]) -> Self {
        Self::from_slice(slice)
    }
}

impl<T, const A: usize> From<Vec<T>> for Buffer<T, A>
where
    T: Primitive,
{
    fn from(vec: Vec<T>) -> Self {
        Self::from_slice(&vec)
    }
}

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
                let mut ptr = unsafe { alloc::<T, A>(layout::<T, A>(lower_bound + 1)) };

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
                    ptr = unsafe { realloc::<T, A, A>(ptr, len, len + 1) };
                    unsafe { ptr.add(len).write(value) };
                    len += 1;
                }

                Self {
                    ptr: unsafe { NonNull::new_unchecked(ptr) },
                    len,
                }
            }
            None => Self::empty(),
        }
    }
}

impl<'a, T, const A: usize> FromIterator<&'a T> for Buffer<T, A>
where
    T: Primitive + 'a,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a T>,
    {
        iter.into_iter().copied().collect()
    }
}

impl<'a, T, const A: usize> IntoIterator for &'a Buffer<T, A>
where
    // Copy instead of Primitive because this is also used in BitMap with usize
    // which does not impl Primitive.
    T: Copy,
{
    type Item = T;
    type IntoIter = Copied<Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref().iter().copied()
    }
}

impl<T, const A: usize> PartialEq for Buffer<T, A>
where
    T: Copy,
    for<'a> &'a [T]: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len
            && (self.len == 0 || (self.layout() == other.layout() && &self[..] == &other[..]))
    }
}

impl<T, const A: usize> Eq for Buffer<T, A>
where
    T: Copy,
    for<'a> &'a [T]: PartialEq,
{
}

/// Returns the [Layout] for a [Buffer] with alignment `A` and provided length
/// (number of elements).
pub(crate) fn layout<T: Copy, const A: usize>(length: usize) -> Layout {
    assert!(length != 0, "Zero-sized layouts are not supported");

    // Power-of-two alignment.
    let align = 1 << A;

    // Make sure the alignment is correct.
    assert!(
        align % mem::align_of::<T>() == 0,
        "Alignment `A` must be a multiple of the ABI-required minimum alignment of type `T`"
    );

    // No additional padding between elements. Buffer types are compatible with
    // the power-of-two alignment.
    let size = length * mem::size_of::<T>();

    // Taken from `padding_needed_for` (requires `const_alloc_layout`).
    let padding =
        (size.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1)).wrapping_sub(size);

    // Rounded up length is the size with padding.
    let (rounded_up_len, overflow) = size.overflowing_add(padding);

    // The rounded up length should not overflow.
    assert!(!overflow, "Allocation size overflow");

    // Safety
    // - Align is non-zero because `A` is the exponent of the power-of-two.
    // - Rounded up length does not overflow, checked above.
    unsafe { Layout::from_size_align_unchecked(rounded_up_len, align) }
}

/// Allocates the memory for a [Buffer] with alignment `A` and provided length
/// (number of elements).
///
/// This method will likely be deprecated when Allocator APIs are stabilized.
pub(crate) unsafe fn alloc<T, const A: usize>(layout: Layout) -> *mut T
where
    T: Copy,
{
    // Safety
    // - Size condition is checked in `layout` function.
    // - Alignment condition is checked in `layout` function.
    let ptr = alloc::alloc(layout) as *mut T;

    // Make sure the allocation did not fail.
    assert!(!ptr.is_null(), "Allocation failed");

    // Return the pointer.
    ptr
}

/// Attempt to reallocate the memory for a [Buffer] with alignment `A` so that
/// it can hold the new length (number of elements) in a buffer with alignment
/// `B`.
///
/// When the layout for the new length matches the layout of the old length,
/// this does not allocate and simply returns the given ptr, because the
/// padding from the previous allocation can hold the required new length.
///
/// When the current allocation can't hold the new length, a new allocation
/// is attempted with the new layout. The values from the previous allocation
/// are copied from the source, and the source location is deallocated.
///
/// # Safety
/// This method is unsafe and its behavior is undefined unless the following
/// conditions are met:
/// todo(mb)
pub(crate) unsafe fn realloc<T, const A: usize, const B: usize>(
    ptr: *mut T,
    old_length: usize,
    new_length: usize,
) -> *mut T
where
    T: Copy,
{
    let old_layout = layout::<T, A>(old_length);
    let new_layout = layout::<T, B>(new_length);

    // Check if the current allocation layout can hold the new length.
    if old_layout == new_layout {
        // No need to reallocate. There is enough capacity to store the new
        // length.
        ptr
    } else {
        // Allocate new buffer and copy contents from source.
        let new_ptr = alloc::<T, B>(new_layout);
        ptr::copy_nonoverlapping(ptr as *const T, new_ptr, old_length);

        // Deallocate previous allocation.
        alloc::dealloc(ptr as *mut u8, old_layout);

        // Return the new pointer.
        new_ptr
    }
}

/// Default exponent of power-of-two alignment for buffers.
pub const ALIGNMENT: usize = 6;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Zero-sized layouts are not supported")]
    fn layout_zero_sized() {
        layout::<u8, 0>(0).size();
    }

    #[test]
    #[should_panic(expected = "Allocation size overflow")]
    fn layout_overflow() {
        layout::<u8, 6>(usize::MAX - 62);
    }

    #[test]
    #[should_panic(
        expected = "Alignment `A` must be a multiple of the ABI-required minimum alignment of type `T`"
    )]
    fn layout_bad_align() {
        layout::<u64, 0>(1234);
    }

    #[test]
    fn layout_size() {
        assert_eq!(layout::<u8, 0>(1).size(), 1);
        assert_eq!(layout::<u8, 5>(1).size(), 32);
        assert_eq!(layout::<u8, 5>(32).size(), 32);
        assert_eq!(layout::<u8, 5>(33).size(), 64);
        assert_eq!(layout::<u8, 6>(1).size(), 64);
        assert_eq!(layout::<u8, 6>(64).size(), 64);
        assert_eq!(layout::<u8, 6>(65).size(), 128);
        assert_eq!(layout::<u32, 6>(5).size(), 64);
        assert_eq!(layout::<f64, 6>(8).size(), 64);
        assert_eq!(layout::<f64, 6>(9).size(), 128);
    }

    #[test]
    fn as_ref() {
        let buffer: Buffer<_, 7> = [1u32, 2, 3, 4].into();
        let x: &Buffer<_, 7> = buffer.as_ref();
        assert_eq!(x.len(), 4);
        let x: &[u8] = buffer.as_ref();
        assert_eq!(x.len(), 4 * 4);
    }

    #[test]
    fn as_ref_u8() {
        let vec = vec![42u32, u32::MAX, 0xc0fefe];
        let buffer: Buffer<_, 7> = vec.into();
        assert_eq!(
            AsRef::<[u8]>::as_ref(&buffer),
            &[42u8, 0, 0, 0, 255, 255, 255, 255, 254, 254, 192, 0]
        );
    }

    #[test]
    fn borrow() {
        let buffer: Buffer<_, 7> = [1u32, 2, 3, 4].into();

        fn borrow_u32<T: Borrow<[u32]>>(input: T) {
            assert_eq!(input.borrow(), &[1, 2, 3, 4]);
        }

        borrow_u32(buffer);
    }

    #[test]
    fn deref() {
        let buffer: Buffer<_, 3> = [1u32, 2, 3, 4].into();
        assert_eq!(buffer.len(), 4);
        assert_eq!(&buffer[2..], &[3, 4]);
    }

    #[test]
    fn empty() {
        let buffer: Buffer<u8, 6> = Buffer::empty();
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
    fn from_array() {
        let array = [1u64, 2, 3, 4];
        let buffer: Buffer<_, 3> = array.into();
        assert_eq!(array, &buffer[..]);
    }

    #[test]
    fn from_boxed_slice() {
        let boxed_slice = vec![1u8, 2, 3, 4].into_boxed_slice();
        let buffer: Buffer<_, 1> = boxed_slice.clone().into();
        assert_eq!(&boxed_slice[..], &buffer[..]);
    }

    #[test]
    fn from_slice() {
        let slice: &[u8] = &[1u8, 2, 3, 4];
        let buffer: Buffer<_, 5> = slice.into();
        assert_eq!(slice, &buffer[..]);
    }

    #[test]
    fn from_vec() {
        let vec = vec![1u8, 2, 3, 4];
        let buffer: Buffer<_, 1> = vec.clone().into();
        assert_eq!(vec, &buffer[..]);
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
        let buffer = vec.iter().collect::<Buffer<_, 4>>();
        assert_eq!(buffer.len(), 4);
        assert_eq!(&vec[..], &buffer[..]);
    }

    #[test]
    fn into_iter() {
        let vec = vec![1u32, 2, 3, 4];
        let other = vec
            .iter()
            .collect::<Buffer<_, 5>>()
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(vec, other);
    }
}
