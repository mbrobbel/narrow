//! Fixed-size types.

use std::mem;

use crate::collection::Item;

/// Fixed-size types.
pub trait FixedSize: Item + Copy {
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

/// Implements `Item` and `ItemMut` for primitive types.
macro_rules! item_primitive {
    ($ty:ty) => {
        impl Item for $ty {
            type Ref<'collection> = Self;
            fn as_ref(&self) -> Self::Ref<'_> {
                *self
            }
            fn to_owned(ref_item: &Self::Ref<'_>) -> Self {
                *ref_item
            }
            fn into_owned(ref_item: Self::Ref<'_>) -> Self {
                ref_item
            }
        }
    };
}

item_primitive!(u8);
item_primitive!(u16);
item_primitive!(u32);
item_primitive!(u64);
item_primitive!(u128);
item_primitive!(usize);

item_primitive!(i8);
item_primitive!(i16);
item_primitive!(i32);
item_primitive!(i64);
item_primitive!(i128);
item_primitive!(isize);

item_primitive!(f32);
item_primitive!(f64);

// bools are stored as bits in collections, so we can't borrow it directly
// instead we return a copy
impl Item for bool {
    type Ref<'collection> = Self;
    fn as_ref(&self) -> Self::Ref<'_> {
        *self
    }
    fn to_owned(ref_item: &Self::Ref<'_>) -> Self {
        *ref_item
    }
    fn into_owned(ref_item: Self::Ref<'_>) -> Self {
        ref_item
    }
}

impl<const N: usize, T: Item + Clone> Item for [T; N] {
    type Ref<'collection> = &'collection [T; N];
    fn as_ref(&self) -> Self::Ref<'_> {
        self
    }
    fn to_owned(ref_item: &Self::Ref<'_>) -> Self {
        (*ref_item).clone()
    }
    fn into_owned(ref_item: Self::Ref<'_>) -> Self {
        ref_item.clone()
    }
}
