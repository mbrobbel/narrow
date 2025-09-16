//! Fixed-size types.

use std::mem;

use crate::collection::AsView;

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

impl<T: FixedSize, const N: usize> FixedSize for [T; N] {}

mod sealed {
    pub trait Sealed {}
    impl<T: super::FixedSize> Sealed for T {}
}

impl<'a, T: FixedSize> AsView<'a> for T {
    type View = T;

    fn as_view(&'a self) -> T {
        *self
    }
}
