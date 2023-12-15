//! Validity trait for nullable and non-nullable data.

use crate::{buffer::BufferType, nullable::Nullable};

/// Validity trait for nullable and non-nullable data.
///
/// This trait has an associated type for storage that is `T` when `NULLABLE` is
/// `false` and [`Nullable<T, Buffer>`] when `NULLABLE` is `true`. In other
/// words, this trait allows wrapping storage types in a [`Nullable`], adding a
/// [`Bitmap`](crate::bitmap::Bitmap) that stores validity information, depending
/// on the const generic `NULLABLE`.
pub trait Validity<const NULLABLE: bool> {
    /// Storage type constructor for data.
    ///
    /// Generic over a [`BufferType`].
    type Storage<Buffer: BufferType>;
}

impl<T> Validity<false> for T {
    type Storage<Buffer: BufferType> = T;
}

impl<T> Validity<true> for T {
    type Storage<Buffer: BufferType> = Nullable<T, Buffer>;
}

/// Nullability trait for nullable and non-nullable items.
pub trait Nullability<const NULLABLE: bool> {
    /// The item, `T` when `NULLABLE` is false, `Option<Item>` when
    /// `NULLABLE` is true.
    type Item;
}

impl<T> Nullability<false> for T {
    type Item = T;
}

impl<T> Nullability<true> for T {
    type Item = Option<T>;
}
