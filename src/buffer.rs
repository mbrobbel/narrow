//! Buffers for data.

use std::{
    borrow::{Borrow, BorrowMut},
    fmt::Debug,
    marker::PhantomData,
    rc::Rc,
    sync::Arc,
};

use crate::{collection::Collection, fixed_size::FixedSize};

/// A contiguous immutable buffer.
pub trait Buffer<T: FixedSize>: Borrow<[T]> + Debug {
    /// Returns a slice containing all the items in this buffer.
    fn as_slice(&self) -> &[T] {
        self.borrow()
    }
}

impl<T: FixedSize, U: Borrow<[T]> + Debug> Buffer<T> for U {}

/// A contiguous mutable buffer.
pub trait BufferMut<T: FixedSize>: Buffer<T> + BorrowMut<[T]> {
    /// Returns a mutable slice containing all the items in this buffer.
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.borrow_mut()
    }
}

impl<T: FixedSize, U: BorrowMut<[T]> + Debug> BufferMut<T> for U {}

/// A [`Buffer`] constructor for [`FixedSize`] types.
pub trait BufferType: Default + Debug {
    /// A [`Buffer`] for [`FixedSize`] items of type `T`.
    type Buffer<T: FixedSize>: Buffer<T> + Collection<Owned = T>;
}

/// A [`BufferType`] for an array with `N` elements.
///
/// Implements [`Buffer`] and [`BufferMut`].
#[derive(Clone, Copy, Default, Debug)]
pub struct ArrayBuffer<const N: usize>;

impl<const N: usize> BufferType for ArrayBuffer<N> {
    type Buffer<T: FixedSize> = [T; N];
}

/// A [`BufferType`] for a [`Vec`].
///
/// Implements [`Buffer`], [`BufferMut`].
#[derive(Clone, Copy, Default, Debug)]
pub struct VecBuffer;

impl BufferType for VecBuffer {
    type Buffer<T: FixedSize> = Vec<T>;
}

/// A [`BufferType`] for a [`Box`]-ed slice.
///
/// Implements [`Buffer`].
#[derive(Clone, Copy, Default, Debug)]
pub struct BoxBuffer;

impl BufferType for BoxBuffer {
    type Buffer<T: FixedSize> = Box<[T]>;
}

/// A [`BufferType`] for an [`Rc`]-ed slice.
///
/// Implements [`Buffer`].
#[derive(Clone, Copy, Default, Debug)]
pub struct RcBuffer;

impl BufferType for RcBuffer {
    type Buffer<T: FixedSize> = Rc<[T]>;
}

/// A [`BufferType`] for an [`Arc`]-ed slice.
///
/// Implements [`Buffer`].
#[derive(Clone, Copy, Default, Debug)]
pub struct ArcBuffer;

impl BufferType for ArcBuffer {
    type Buffer<T: FixedSize> = Arc<[T]>;
}

/// A [`BufferType`] for a slice.
///
/// Implements [`Buffer`].
#[derive(Clone, Copy, Default, Debug)]
pub struct SliceBuffer<'slice>(PhantomData<&'slice ()>);

impl<'slice> BufferType for SliceBuffer<'slice> {
    type Buffer<T: FixedSize> = &'slice [T];
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

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

        assert!(HasBuffer::<<VecBuffer as BufferType>::Buffer<u16>, _>::IMPL);
        assert!(HasBufferMut::<<VecBuffer as BufferType>::Buffer<u16>, _>::IMPL);

        assert!(HasBuffer::<<ArrayBuffer<1> as BufferType>::Buffer<u16>, _>::IMPL);
        assert!(HasBufferMut::<<ArrayBuffer<1> as BufferType>::Buffer<u16>, _>::IMPL);

        assert!(HasBuffer::<<SliceBuffer<'_> as BufferType>::Buffer<u16>, _>::IMPL);

        assert!(HasBuffer::<<BoxBuffer as BufferType>::Buffer<u16>, _>::IMPL);

        assert!(HasBuffer::<<RcBuffer as BufferType>::Buffer<u16>, _>::IMPL);

        assert!(HasBuffer::<<ArcBuffer as BufferType>::Buffer<u16>, _>::IMPL);
    }
}
