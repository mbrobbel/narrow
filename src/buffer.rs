//! Traits for memory buffers.

use std::{
    borrow::{Borrow, BorrowMut},
    mem,
    rc::Rc,
    slice,
    sync::Arc,
};

use crate::Primitive;

/// A contiguous immutable memory buffer for data.
///
/// Read-only slice.
///
/// There is a blanket implementation, so that every type implements
/// `Borrow<[T]> where T: Primitive` can be used as a `Buffer` in this
/// crate.
pub trait Buffer<T>
where
    T: Primitive,
    Self: Borrow<[T]>,
{
    fn as_bytes(&self) -> &[u8] {
        // Safety:
        // - The pointer returned by slice::as_ptr (via Borrow) points to slice::len()
        //   consecutive properly initialized values of type T, with size_of::<T> bytes
        //   per element.
        unsafe {
            slice::from_raw_parts(
                self.borrow().as_ptr() as *const u8,
                self.borrow().len() * mem::size_of::<T>(),
            )
        }
    }
}

// Any type that can be borrowed as a slice of some `Primitive` can be used as a
// Buffer.
impl<T, U: ?Sized> Buffer<T> for U
where
    T: Primitive,
    U: Borrow<[T]>,
{
}

/// A contiguous mutable memory buffer for data.
///
/// In-place mutation.
pub trait BufferMut<T>
where
    T: Primitive,
    Self: Buffer<T> + BorrowMut<[T]>,
{
}

// Any type that can be borrowed as a mutable slice of some `Primitive` can be
// used as a BufferMut.
impl<T, U: ?Sized> BufferMut<T> for U
where
    T: Primitive,
    U: Buffer<T> + BorrowMut<[T]>,
{
}

/// An allocatable contiguous memory buffer for data.
///
/// Allocation.
pub trait BufferAlloc<T>
where
    T: Primitive,
    Self: Buffer<T> + FromIterator<T>,
{
}

impl<T> BufferAlloc<T> for Vec<T> where T: Primitive {}
impl<T> BufferAlloc<T> for Box<[T]> where T: Primitive {}
impl<T> BufferAlloc<T> for Rc<[T]> where T: Primitive {}
impl<T> BufferAlloc<T> for Arc<[T]> where T: Primitive {}

/// An extendable contiguous memory buffer for data.
///
/// Growing and shrinking.
pub trait BufferExtend<T>
where
    T: Primitive,
    Self: BufferMut<T> + Extend<T>,
{
}

impl<T, U> BufferExtend<T> for U
where
    T: Primitive,
    U: BufferMut<T> + Extend<T>,
{
}

/// A buffer that can be consumed via [IntoIterator].
pub trait BufferTake<T>
where
    T: Primitive,
    Self: Buffer<T> + IntoIterator<Item = T>,
{
}

impl<T, U> BufferTake<T> for U
where
    T: Primitive,
    U: Buffer<T> + IntoIterator<Item = T>,
{
}

/// A reference to a buffer.
pub trait BufferRef {
    type Element: Primitive;
    type Buffer: ?Sized + Buffer<Self::Element>;

    /// Returns a reference to the buffer.
    fn buffer_ref(&self) -> &Self::Buffer;
}

impl<T> BufferRef for Vec<T>
where
    T: Primitive,
{
    type Buffer = [T];
    type Element = T;

    fn buffer_ref(&self) -> &Self::Buffer {
        self.as_slice()
    }
}

impl<T, const N: usize> BufferRef for Vec<[T; N]>
where
    T: Primitive,
{
    type Buffer = [T];
    type Element = T;

    fn buffer_ref(&self) -> &Self::Buffer {
        // self.flatten() is nightly
        // SAFETY: `[T]` is layout-identical to `[T; N]`
        unsafe { std::slice::from_raw_parts(self.as_ptr().cast(), self.len() * N) }
    }
}

/// A mutable reference to a mutable buffer.
pub trait BufferRefMut {
    type Element: Primitive;
    type BufferMut: ?Sized + BufferMut<Self::Element>;

    /// Returns a mutable reference to the mutable buffer.
    fn buffer_ref_mut(&mut self) -> &mut Self::BufferMut;
}

impl<T> BufferRefMut for Vec<T>
where
    T: Primitive,
{
    type BufferMut = [T];
    type Element = T;

    fn buffer_ref_mut(&mut self) -> &mut Self::BufferMut {
        self
    }
}

impl<T, const N: usize> BufferRefMut for Vec<[T; N]>
where
    T: Primitive,
{
    type BufferMut = [T];
    type Element = T;

    fn buffer_ref_mut(&mut self) -> &mut Self::BufferMut {
        // self.flatten() is nightly
        // SAFETY: `[T]` is layout-identical to `[T; N]`
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr().cast(), self.len() * N) }
    }
}
