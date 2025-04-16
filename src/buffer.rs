//! Buffers for data.

use std::{
    borrow::{Borrow, BorrowMut},
    marker::PhantomData,
    rc::Rc,
    sync::Arc,
};

use crate::{
    collection::{Collection, CollectionAlloc, CollectionMut},
    fixed_size::FixedSize,
};

/// A [`Buffer`] constructor for [`FixedSize`] types.
pub trait BufferType {
    /// A [`Buffer`] for [`FixedSize`] items of type `T`.
    type Buffer<T: FixedSize>: Buffer<T>;
}

/// A [`BufferType`] for [`[T; N]`].
///
/// Implements [`Buffer`] and [`BufferMut`].
#[derive(Clone, Copy, Default)]
pub struct ArrayBuffer<const N: usize>;

impl<const N: usize> BufferType for ArrayBuffer<N> {
    type Buffer<T: FixedSize> = [T; N];
}

/// A [`BufferType`] for [`Vec<T>`].
///
/// Implements [`Buffer`], [`BufferMut`] and [`BufferAlloc`].
#[derive(Clone, Copy, Default)]
pub struct VecBuffer;

impl BufferType for VecBuffer {
    type Buffer<T: FixedSize> = Vec<T>;
}

/// A [`BufferType`] for [`&[T]`].
///
/// Implements [`Buffer`].
#[derive(Clone, Copy, Default)]
pub struct SliceBuffer<'slice>(PhantomData<&'slice ()>);

impl<'slice> BufferType for SliceBuffer<'slice> {
    type Buffer<T: FixedSize> = &'slice [T];
}

/// A [`BufferType`] for [`&mut [T]`].
///
/// Implements [`Buffer`] and [`BufferMut`].
#[derive(Clone, Copy, Default)]
pub struct SliceMutBuffer<'slice>(PhantomData<&'slice ()>);

impl<'slice> BufferType for SliceMutBuffer<'slice> {
    type Buffer<T: FixedSize> = &'slice mut [T];
}

/// A [`BufferType`] for [`Box<[T]>`].
///
/// Implements [`Buffer`].
#[derive(Clone, Copy, Default)]
pub struct BoxBuffer;

impl BufferType for BoxBuffer {
    type Buffer<T: FixedSize> = Box<[T]>;
}

/// A [`BufferType`] for [`Rc<[T]>`].
///
/// Implements [`Buffer`].
#[derive(Clone, Copy, Default)]
pub struct RcBuffer;

impl BufferType for RcBuffer {
    type Buffer<T: FixedSize> = Rc<[T]>;
}

/// A [`BufferType`] for [`Arc<[T]>`].
///
/// Implements [`Buffer`].
#[derive(Clone, Copy, Default)]
pub struct ArcBuffer;

impl BufferType for ArcBuffer {
    type Buffer<T: FixedSize> = Arc<[T]>;
}

/// A contiguous immutable buffer.
pub trait Buffer<T: FixedSize>: Borrow<[T]> + Collection<Item = T> {
    /// Returns a slice containing all the items in this buffer.
    fn as_slice(&self) -> &[T] {
        self.borrow()
    }
}

impl<T, U> Buffer<T> for U
where
    T: FixedSize,
    U: Borrow<[T]> + Collection<Item = T>,
{
}

/// A contiguous mutable buffer.
pub trait BufferMut<T: FixedSize>: Buffer<T> + BorrowMut<[T]> + CollectionMut<Item = T> {
    /// Returns a mutable slice containing all the items in this buffer.
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.borrow_mut()
    }
}

impl<T, U> BufferMut<T> for U
where
    T: FixedSize,
    U: BorrowMut<[T]> + CollectionMut<Item = T>,
{
}

/// An allocatable contiguous buffer.
pub trait BufferAlloc<T: FixedSize>: Buffer<T> + CollectionAlloc<Item = T> {}

impl<T, U> BufferAlloc<T> for U
where
    T: FixedSize,
    U: Buffer<T> + CollectionAlloc<Item = T>,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn buffer_types() {
        struct HasBuffer<U, T>(PhantomData<(T, U)>);
        impl<T: FixedSize, U: Buffer<T>> HasBuffer<U, T> {
            const IMPL: bool = true;
        }
        struct HasBufferMut<U, T>(PhantomData<(T, U)>);
        impl<T: FixedSize, U: BufferMut<T>> HasBufferMut<U, T> {
            const IMPL: bool = true;
        }
        struct HasBufferAlloc<U, T>(PhantomData<(T, U)>);
        impl<T: FixedSize, U: BufferAlloc<T>> HasBufferAlloc<U, T> {
            const IMPL: bool = true;
        }

        assert!(HasBuffer::<<VecBuffer as BufferType>::Buffer<u16>, _>::IMPL);
        assert!(HasBufferMut::<<VecBuffer as BufferType>::Buffer<u16>, _>::IMPL);
        assert!(HasBufferAlloc::<<VecBuffer as BufferType>::Buffer<u16>, _>::IMPL);

        assert!(HasBuffer::<<ArrayBuffer<1> as BufferType>::Buffer<u16>, _>::IMPL);
        assert!(HasBufferMut::<<ArrayBuffer<1> as BufferType>::Buffer<u16>, _>::IMPL);

        assert!(HasBuffer::<<SliceMutBuffer<'_> as BufferType>::Buffer<u16>, _>::IMPL);
        assert!(HasBufferMut::<<SliceMutBuffer<'_> as BufferType>::Buffer<u16>, _>::IMPL);

        assert!(HasBuffer::<<SliceBuffer<'_> as BufferType>::Buffer<u16>, _>::IMPL);

        assert!(HasBuffer::<<BoxBuffer as BufferType>::Buffer<u16>, _>::IMPL);

        assert!(HasBuffer::<<RcBuffer as BufferType>::Buffer<u16>, _>::IMPL);

        assert!(HasBuffer::<<ArcBuffer as BufferType>::Buffer<u16>, _>::IMPL);
    }
}
