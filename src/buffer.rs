//! Traits for memory buffers.

use std::{
    borrow::{Borrow, BorrowMut},
    mem, slice,
};

use crate::Primitive;

/// A contiguous immutable memory buffer for data.
///
/// Read-only slice.
///
/// There is a blanket implementation, so that everything that implements
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

impl<T, U> Buffer<T> for U
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
    Self: Buffer<T>,
    Self: BorrowMut<[T]>,
{
}

impl<T, U> BufferMut<T> for U
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
    Self: Buffer<T>,
    Self: FromIterator<T>,
{
}

impl<T> BufferAlloc<T> for Vec<T> where T: Primitive {}
impl<T> BufferAlloc<T> for Box<[T]> where T: Primitive {}

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
