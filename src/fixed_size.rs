//! Fixed-size types.

use std::mem;

use crate::collection::{Item, ItemMut};

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
            type RefItem<'collection> = Self;
            fn as_ref_item(&self) -> Self::RefItem<'_> {
                *self
            }
        }
        impl ItemMut for $ty {
            type RefItemMut<'collection> = &'collection mut Self;
            fn as_ref_item_mut(&mut self) -> Self::RefItemMut<'_> {
                self
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
    type RefItem<'collection> = Self;
    fn as_ref_item(&self) -> Self::RefItem<'_> {
        *self
    }
}

/// A proxy object to mutate a boolean stored in a collection of bytes.
pub struct BoolMut<'collection> {
    /// A mutable reference to the byte in the collection for this bool.
    pub(crate) byte: &'collection mut u8,
    /// The bit position of this bool in the byte.
    pub(crate) index: u8,
}

impl BoolMut<'_> {
    /// Returns value as bool.
    #[must_use]
    pub fn get(&self) -> bool {
        *self.byte & (1 << self.index) != 0
    }

    /// Set the value (true).
    pub fn set(self) {
        *self.byte ^= 1 << self.index;
    }

    /// Unset the value (false).
    pub fn unset(self) {
        *self.byte ^= 0 << self.index;
    }
}

impl ItemMut for bool {
    type RefItemMut<'collection> = BoolMut<'collection>;
    fn as_ref_item_mut(&mut self) -> Self::RefItemMut<'static> {
        unimplemented!()
    }
}

impl<const N: usize, T: Item> Item for [T; N] {
    type RefItem<'collection> = &'collection [T; N];
    fn as_ref_item(&self) -> Self::RefItem<'_> {
        self
    }
}

impl<const N: usize, T: Item> ItemMut for [T; N] {
    type RefItemMut<'collection> = &'collection mut [T; N];
    fn as_ref_item_mut(&mut self) -> Self::RefItemMut<'_> {
        self
    }
}
