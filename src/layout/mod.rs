//! Physical memory layouts.

extern crate alloc;

use alloc::vec::Vec;

use crate::{
    buffer::Buffer,
    collection::Collection,
    fixed_size::FixedSize,
    layout::{
        boolean::Boolean, fixed_size_list::FixedSizeList, fixed_size_primitive::FixedSizePrimitive,
        variable_size_list::VariableSizeList,
    },
    nullability::{NonNullable, Nullability, Nullable},
};

pub mod boolean;
pub mod fixed_size_list;
pub mod fixed_size_primitive;
pub mod variable_size_binary;
pub mod variable_size_list;

/// A physical memory layout.
///
/// # Examples
///
/// ```
/// use narrow::layout::{MemoryLayout, fixed_size_primitive::FixedSizePrimitive};
///
/// fn assert_memory_layout<T: MemoryLayout>() {}
/// assert_memory_layout::<FixedSizePrimitive<i32>>();
/// ```
pub trait MemoryLayout: Collection {}

/// Mapping a base type to its physical memory layout.
///
/// # Examples
///
/// ```
/// use narrow::{buffer::VecBuffer, layout::{Layout, boolean::Boolean}, nullability::NonNullable};
///
/// fn assert_layout<T: Layout<Memory<NonNullable, VecBuffer> = Boolean>>() {}
/// assert_layout::<bool>();
/// ```
pub trait Layout: Sized {
    /// The Arrow physical memory layout of this type.
    type Memory<Nulls: Nullability, Storage: Buffer>: MemoryLayout<Owned = Nulls::Item<Self>>;
}

/// Marker for base types whose layout supports Arrow validity bitmaps.
///
/// # Examples
///
/// ```
/// use narrow::layout::NullableLayout;
///
/// fn assert_nullable<T: NullableLayout>() {}
/// assert_nullable::<Vec<i32>>();
/// ```
pub trait NullableLayout: Layout {}

/// Mapping an array item type to its complete physical memory layout.
///
/// # Examples
///
/// ```
/// use narrow::{buffer::VecBuffer, layout::{ArrayItem, boolean::Boolean}, nullability::Nullable};
///
/// fn assert_item<T: ArrayItem<Memory<VecBuffer> = Boolean<Nullable>>>() {}
/// assert_item::<Option<bool>>();
/// ```
pub trait ArrayItem: Sized {
    /// The Arrow physical memory layout of this array item type.
    type Memory<Storage: Buffer>: MemoryLayout<Owned = Self>;
}

impl<T: Layout> ArrayItem for T {
    type Memory<Storage: Buffer> = T::Memory<NonNullable, Storage>;
}

impl<T: NullableLayout> ArrayItem for Option<T> {
    type Memory<Storage: Buffer> = T::Memory<Nullable, Storage>;
}

impl Layout for bool {
    type Memory<Nulls: Nullability, Storage: Buffer> = Boolean<Nulls, Storage>;
}

impl NullableLayout for bool {}

impl<T: FixedSize> Layout for T {
    type Memory<Nulls: Nullability, Storage: Buffer> = FixedSizePrimitive<T, Nulls, Storage>;
}

impl<T: FixedSize> NullableLayout for T {}

impl<T: ArrayItem, const N: usize> Layout for [T; N] {
    type Memory<Nulls: Nullability, Storage: Buffer> = FixedSizeList<T, N, Nulls, Storage>;
}

impl<T: ArrayItem, const N: usize> NullableLayout for [T; N] {}

impl<T: ArrayItem> Layout for Vec<T> {
    type Memory<Nulls: Nullability, Storage: Buffer> = VariableSizeList<T, Nulls, i32, Storage>;
}

impl<T: ArrayItem> NullableLayout for Vec<T> {}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec::Vec;

    use crate::{
        buffer::VecBuffer,
        layout::{
            ArrayItem, boolean::Boolean, fixed_size_list::FixedSizeList,
            fixed_size_primitive::FixedSizePrimitive, variable_size_list::VariableSizeList,
        },
        nullability::{NonNullable, Nullable},
    };

    fn assert_memory<T: ArrayItem<Memory<VecBuffer> = Memory>, Memory>() {}

    #[test]
    fn array_item_selects_nullability() {
        assert_memory::<bool, Boolean<NonNullable, VecBuffer>>();
        assert_memory::<Option<bool>, Boolean<Nullable, VecBuffer>>();
        assert_memory::<i32, FixedSizePrimitive<i32, NonNullable, VecBuffer>>();
        assert_memory::<Option<i32>, FixedSizePrimitive<i32, Nullable, VecBuffer>>();
        assert_memory::<[Option<i32>; 2], FixedSizeList<Option<i32>, 2>>();
        assert_memory::<Option<[Option<i32>; 2]>, FixedSizeList<Option<i32>, 2, Nullable>>();
        assert_memory::<Vec<Option<i32>>, VariableSizeList<Option<i32>>>();
        assert_memory::<Option<Vec<Option<i32>>>, VariableSizeList<Option<i32>, Nullable>>();
    }
}
