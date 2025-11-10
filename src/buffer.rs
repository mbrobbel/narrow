//! Contiguous collections for fixed size items.

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};
use core::{borrow::Borrow, marker::PhantomData};

use crate::{collection::Collection, fixed_size::FixedSize};

/// Constructor for contiguous [`Collection`]s of [`FixedSize`] items.
pub trait Buffer: Default {
    /// The [`Collection`] with [`FixedSize`] items.
    /// Can be borrowed as a contiguous slice of `T`.
    type For<T: FixedSize>: Borrow<[T]> + Collection<Owned = T>;
}

#[derive(Clone, Copy, Default, Debug)]
pub struct ArrayBuffer<const N: usize>;
impl<const N: usize> Buffer for ArrayBuffer<N> {
    type For<T: FixedSize> = [T; N];
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VecBuffer;
impl Buffer for VecBuffer {
    type For<T: FixedSize> = Vec<T>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BoxBuffer;
impl Buffer for BoxBuffer {
    type For<T: FixedSize> = Box<[T]>;
}

#[derive(Clone, Copy, Default, Debug)]
pub struct RcBuffer;
impl Buffer for RcBuffer {
    type For<T: FixedSize> = Rc<[T]>;
}

#[derive(Clone, Copy, Default, Debug)]
pub struct ArcBuffer;
impl Buffer for ArcBuffer {
    type For<T: FixedSize> = Arc<[T]>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SliceBuffer<'slice>(PhantomData<&'slice ()>);
impl<'slice> Buffer for SliceBuffer<'slice> {
    type For<T: FixedSize> = &'slice [T];
}
