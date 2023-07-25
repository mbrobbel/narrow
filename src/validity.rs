//! Validity trait for nullable and non-nullable data.

use crate::{buffer::BufferType, nullable::Nullable};

/// Validity trait for nullable and non-nullable data.
///
/// This trait has an associated type for storage that is `T` when `NULLABLE` is
/// `false` and `Nullable<T, Buffer>` when `NULLABLE` is `true`. In other
/// words, this trait allows wrapping storage types in a `Nullable`, basically
/// adding a [Bitmap](crate::bitmap::Bitmap) that stores validity information,
/// depending on the const generic `NULLABLE`.
pub trait Validity<const NULLABLE: bool> {
    /// Storage type constructor for data.
    ///
    /// Generic over a [Bitmap](crate::bitmap::Bitmap)'s [BufferType].
    type Storage<BitmapBuffer: BufferType>;
}

impl<T> Validity<false> for T {
    type Storage<BitmapBuffer: BufferType> = T;
}

impl<T> Validity<true> for T {
    type Storage<BitmapBuffer: BufferType> = Nullable<T, BitmapBuffer>;
}
