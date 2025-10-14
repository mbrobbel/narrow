//! Physical memory layouts.

use crate::{
    buffer::Buffer,
    collection::Collection,
    fixed_size::FixedSize,
    layout::{
        fixed_size_list::FixedSizeList, fixed_size_primitive::FixedSizePrimitive,
        variable_size_list::VariableSizeList,
    },
    nullability::{NonNullable, Nullable},
};

pub mod fixed_size_list;
pub mod fixed_size_primitive;
pub mod variable_size_binary;
pub mod variable_size_list;

/// A physical memory layout.
pub trait MemoryLayout: Collection {}

/// Mapping types to their physical memory layout.
pub trait Layout {
    /// The Arrow physical memory layout of this type.
    type Memory<Storage: Buffer>: MemoryLayout<Owned = Self>;
}

impl<T: FixedSize> Layout for T {
    type Memory<Storage: Buffer> = FixedSizePrimitive<T, NonNullable, Storage>;
}

impl<T: FixedSize> Layout for Option<T> {
    type Memory<Storage: Buffer> = FixedSizePrimitive<T, Nullable, Storage>;
}

impl<T: Layout, const N: usize> Layout for [T; N] {
    type Memory<Storage: Buffer> = FixedSizeList<T, N, NonNullable, Storage>;
}

impl<T: Layout, const N: usize> Layout for Option<[T; N]> {
    type Memory<Storage: Buffer> = FixedSizeList<T, N, Nullable, Storage>;
}

impl<T: Layout> Layout for Vec<T> {
    type Memory<Storage: Buffer> = VariableSizeList<T, NonNullable, i32, Storage>;
}

impl<T: Layout> Layout for Option<Vec<T>> {
    type Memory<Storage: Buffer> = VariableSizeList<T, Nullable, i32, Storage>;
}
