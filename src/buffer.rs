//! Contiguous collections for fixed size items.

extern crate alloc;

#[cfg(target_has_atomic = "ptr")]
use alloc::sync::Arc;
use alloc::{boxed::Box, rc::Rc, vec::Vec};
use core::{borrow::Borrow, marker::PhantomData};

use crate::{collection::Collection, fixed_size::FixedSize};

/// Constructor for contiguous [`Collection`]s of [`FixedSize`] items.
///
/// Arrow describes buffer contents, not how those buffers are owned. `Buffer`
/// keeps layout types independent of ownership by mapping each element type
/// `T` to a concrete contiguous collection through [`Buffer::For`].
///
/// # Examples
///
/// ```
/// use narrow::buffer::{Buffer, VecBuffer};
///
/// let values: <VecBuffer as Buffer>::For<u16> = vec![1, 2];
/// assert_eq!(values, [1, 2]);
/// ```
pub trait Buffer: Default {
    /// The [`Collection`] with [`FixedSize`] items.
    /// Can be borrowed as a contiguous slice of `T`.
    type For<T: FixedSize>: Borrow<[T]> + Collection<Owned = T>;
}

/// A [`Buffer`] whose collections are empty by default.
///
/// Layouts and [`Array`](crate::array::Array)s implement [`Default`] only for
/// this storage, so their default value never has items. Implementations
/// must make [`Default`] of every [`Buffer::For`] collection return an empty
/// collection.
///
/// [`ArrayBuffer`] does not implement this trait, because its collections
/// always hold `N` values.
///
/// # Examples
///
/// ```
/// use narrow::{array::Array, buffer::ArcBuffer, length::Length};
///
/// let values = Array::<i32, ArcBuffer>::default();
/// assert!(values.is_empty());
/// ```
pub trait EmptyBuffer: Buffer {}

/// Immutable access to a backing buffer or collection.
///
/// Layout wrappers intentionally hide their representation during ordinary
/// collection access. This trait provides a uniform escape hatch for
/// interoperability code that needs the exact Arrow backing buffer.
///
/// # Examples
///
/// ```
/// use narrow::{array::Array, buffer::BufferRef, length::Length};
///
/// let values = [1, 2].into_iter().collect::<Array<i32>>();
/// assert_eq!(values.buffer_ref().len(), 2);
/// ```
pub trait BufferRef {
    /// Backing buffer or collection.
    type Buffer: Collection;

    /// Returns the backing buffer or collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{array::Array, buffer::BufferRef, length::Length};
    ///
    /// let values = [1, 2].into_iter().collect::<Array<i32>>();
    /// assert_eq!(values.buffer_ref().len(), 2);
    /// ```
    fn buffer_ref(&self) -> &Self::Buffer;
}

/// Fixed-length array storage.
///
/// Stores a statically known number of values inline, which is useful when the
/// complete buffer size belongs in the type and no allocation is desired.
///
/// # Examples
///
/// ```
/// use narrow::buffer::{ArrayBuffer, Buffer};
///
/// let values: <ArrayBuffer<2> as Buffer>::For<u16> = [1, 2];
/// assert_eq!(values.len(), 2);
/// ```
///
/// Its collections always hold exactly `N` values, so it is not an
/// [`EmptyBuffer`] and arrays using it have no [`Default`]:
///
/// ```compile_fail,E0599
/// use narrow::{array::Array, buffer::ArrayBuffer};
///
/// let values = Array::<i32, ArrayBuffer<4>>::default();
/// ```
///
/// ```compile_fail,E0599
/// use narrow::{array::Array, buffer::ArrayBuffer};
///
/// let values = Array::<Option<bool>, ArrayBuffer<1>>::default();
/// ```
///
/// ```compile_fail,E0599
/// use narrow::{array::Array, buffer::ArrayBuffer};
///
/// let values = Array::<[i32; 2], ArrayBuffer<3>>::default();
/// ```
///
/// ```compile_fail,E0599
/// use narrow::{array::Array, buffer::ArrayBuffer};
///
/// let values = Array::<(), ArrayBuffer<3>>::default();
/// ```
#[derive(Clone, Copy, Default, Debug)]
pub struct ArrayBuffer<const N: usize>;
impl<const N: usize> Buffer for ArrayBuffer<N> {
    type For<T: FixedSize> = [T; N];
}

/// Growable vector storage.
///
/// This is the default storage because it supports efficient construction and
/// extension while retaining Arrow's required contiguous representation.
///
/// # Examples
///
/// ```
/// use narrow::buffer::{Buffer, VecBuffer};
///
/// let values: <VecBuffer as Buffer>::For<u16> = vec![1, 2];
/// assert_eq!(values.len(), 2);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct VecBuffer;
impl Buffer for VecBuffer {
    type For<T: FixedSize> = Vec<T>;
}
impl EmptyBuffer for VecBuffer {}

/// Owned boxed-slice storage.
///
/// Represents an owned, fixed-capacity buffer without retaining the spare
/// capacity of a [`Vec`].
///
/// # Examples
///
/// ```
/// use narrow::buffer::{BoxBuffer, Buffer};
///
/// let values: <BoxBuffer as Buffer>::For<u16> = Box::from([1, 2]);
/// assert_eq!(&*values, &[1, 2]);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct BoxBuffer;
impl Buffer for BoxBuffer {
    type For<T: FixedSize> = Box<[T]>;
}
impl EmptyBuffer for BoxBuffer {}

/// Reference-counted storage.
///
/// Allows immutable Arrow buffers to be shared within one thread without
/// copying their values.
///
/// # Examples
///
/// ```
/// use std::rc::Rc;
/// use narrow::buffer::{Buffer, RcBuffer};
///
/// let values: <RcBuffer as Buffer>::For<u16> = Rc::from([1, 2]);
/// assert_eq!(&*values, &[1, 2]);
/// ```
#[derive(Clone, Copy, Default, Debug)]
pub struct RcBuffer;
impl Buffer for RcBuffer {
    type For<T: FixedSize> = Rc<[T]>;
}
impl EmptyBuffer for RcBuffer {}

/// Atomically reference-counted storage.
///
/// Allows immutable Arrow buffers to be shared across threads without copying
/// their values.
///
/// Like [`Arc`], this is only available on targets with atomic pointer
/// support (`target_has_atomic = "ptr"`).
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use narrow::buffer::{ArcBuffer, Buffer};
///
/// let values: <ArcBuffer as Buffer>::For<u16> = Arc::from([1, 2]);
/// assert_eq!(&*values, &[1, 2]);
/// ```
#[cfg(target_has_atomic = "ptr")]
#[derive(Clone, Copy, Default, Debug)]
pub struct ArcBuffer;
#[cfg(target_has_atomic = "ptr")]
impl Buffer for ArcBuffer {
    type For<T: FixedSize> = Arc<[T]>;
}
impl EmptyBuffer for ArcBuffer {}

/// Borrowed slice storage.
///
/// Lets a layout view caller-owned contiguous memory. The lifetime in the
/// storage type makes the borrowed ownership model visible to the compiler.
///
/// # Examples
///
/// ```
/// use narrow::buffer::{Buffer, SliceBuffer};
///
/// let source = [1_u16, 2];
/// let values: <SliceBuffer<'_> as Buffer>::For<u16> = &source;
/// assert_eq!(values, &[1, 2]);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct SliceBuffer<'slice>(PhantomData<&'slice ()>);
impl<'slice> Buffer for SliceBuffer<'slice> {
    type For<T: FixedSize> = &'slice [T];
}
impl EmptyBuffer for SliceBuffer<'_> {}
