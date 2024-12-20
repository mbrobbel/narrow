//! Physical memory laouts of Arrow.
//!

use crate::{
    buffer::BufferType, nullability::Collection, FixedSize, Index, Length, NonNullable,
    Nullability, Nullable,
};

/// Well-defined physical layouts of Arrow
pub trait PhysicalLayout: Collection {
    // /// Items in this layout.
    // type Item;

    // /// Reference type for items in this layout.
    // type RefItem<'a>
    // where
    //     Self: 'a;
}

/// An array type.
pub trait Layout {
    /// The layout for this type.
    type PhysicalLayout<Buffer: BufferType>: PhysicalLayout;
}

impl Layout for u8 {
    type PhysicalLayout<Buffer: BufferType> = FixedSizePrimitive<u8, NonNullable, Buffer>;
}
impl Layout for Option<u8> {
    type PhysicalLayout<Buffer: BufferType> = FixedSizePrimitive<u8, Nullable, Buffer>;
}

/// A primitive value array represents an array of values each having the same
/// physical slot width typically measured in bytes, though the spec also
/// provides for bit-packed types (e.g. boolean values encoded in bits).
pub struct FixedSizePrimitive<T: FixedSize, Nullable: Nullability, Buffer: BufferType>(
    Nullable::Collection<Buffer::Buffer<T>, Buffer>,
);

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> Length
    for FixedSizePrimitive<T, Nullable, Buffer>
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> Index
    for FixedSizePrimitive<T, Nullable, Buffer>
{
    type Item<'a>
        = &'a T
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, _index: usize) -> Self::Item<'_> {
        todo!()
    }
}

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> PhysicalLayout
    for FixedSizePrimitive<T, Nullable, Buffer>
{
}

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> Collection
    for FixedSizePrimitive<T, Nullable, Buffer>
{
    type Item = Nullable::Item<T>;
    type RefItem<'a>
        = &'a Nullable::Item<T>
    where
        Self: 'a;

    type Iter<'a>
        = <<Nullable as Nullability>::Collection<<Buffer as BufferType>::Buffer<T>, Buffer> as Collection>::Iter<'a>
    where
        Self: 'a;

    type IntoIter = <<Nullable as Nullability>::Collection<
        <Buffer as BufferType>::Buffer<T>,
        Buffer,
    > as Collection>::IntoIter;

    fn iter(&self) -> Self::Iter<'_> {
        Collection::iter(&self.0)
    }

    fn into_iter(self) -> Self::IntoIter {
        Collection::into_iter(self.0)
    }
}
