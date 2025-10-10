//! Physical memory layouts.

use crate::{
    buffer::Buffer,
    collection::Collection,
    fixed_size::FixedSize,
    layout::{fixed_size_primitive::FixedSizePrimitive, variable_size_binary::VariableSizeBinary},
    nullability::{NonNullable, Nullable},
};

pub mod fixed_size_primitive;
pub mod variable_size_binary;

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

impl Layout for Vec<u8> {
    type Memory<Storage: Buffer> = VariableSizeBinary<NonNullable, i32, Storage>;
}

impl Layout for Option<Vec<u8>> {
    type Memory<Storage: Buffer> = VariableSizeBinary<Nullable, i32, Storage>;
}
