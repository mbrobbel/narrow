//! Fixed-size types.

use std::{mem, ops::Deref};

/// Fixed-size types.
pub trait FixedSize: Copy + sealed::Sealed + 'static {
    /// The size of this type in bytes.
    const SIZE: usize = mem::size_of::<Self>();
}

impl FixedSize for u8 {}
impl FixedSize for u16 {}
impl FixedSize for u32 {}
impl FixedSize for u64 {}
impl FixedSize for u128 {}
impl FixedSize for usize {}

impl FixedSize for i8 {}
impl FixedSize for i16 {}
impl FixedSize for i32 {}
impl FixedSize for i64 {}
impl FixedSize for i128 {}
impl FixedSize for isize {}

impl FixedSize for f32 {}
impl FixedSize for f64 {}

/// An array with `N` `FixedSize` items per item.
///
/// Just using [T; N] causes overlapping impls.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedSizeArray<T: FixedSize, const N: usize>([T; N]);

impl<T: FixedSize, const N: usize> Default for FixedSizeArray<T, N>
where
    [T; N]: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: FixedSize, const N: usize> From<[T; N]> for FixedSizeArray<T, N> {
    fn from(value: [T; N]) -> Self {
        Self(value)
    }
}

impl<T: FixedSize, const N: usize> Deref for FixedSizeArray<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: FixedSize, const N: usize> FixedSize for FixedSizeArray<T, N> {}

mod sealed {
    pub trait Sealed {}
    impl<T: super::FixedSize> Sealed for T {}
}
