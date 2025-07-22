//! Nullable and non-nullable data.

use crate::{buffer::BufferType, collection::Collection, validity::Validity};

/// Nullability trait for nullable and non-nullable type constructors.
///
/// See [`NonNullable`] and [`Nullable`].
pub trait Nullability: sealed::Sealed {
    /// `true` iff this is [`Nullable`].
    const NULLABLE: bool;

    /// Type constructor for nullable and non-nullable items.
    type Item<T>;

    /// Constructor for nullable and non-nullable collections.
    ///
    /// Generic over a collection `T` and a [`BufferType`].
    type Collection<T: Collection, Buffer: BufferType>: Collection<
        Item = Self::Item<<T as Collection>::Item>,
    >;
}

/// Private module for [`sealed::Sealed`] trait.
mod sealed {
    /// Used to seal [`super::Nullability`].
    pub trait Sealed {}

    /// Prevent downstream implementations of [`super::Nullability`].
    impl<T> Sealed for T where T: super::Nullability {}
}

/// Non-nullable types.
///
/// Implements [`Nullability`] to provide:
/// - `NonNullable::Item<T> = T`
/// - `NonNullable::Collection<T, Buffer> = T`
#[derive(Clone, Copy, Debug)]
pub struct NonNullable;

impl Nullability for NonNullable {
    const NULLABLE: bool = false;

    /// Non-nullable items are just `T`.
    type Item<T> = T;

    /// Non-nullable collections are just `T`.
    type Collection<T: Collection, Buffer: BufferType> = T;
}

/// Nullable types.
///
/// Implements [`Nullability`] to provide:
/// - `Nullable::Item<T> = Option<T>`
/// - `Nullable::Collection<T, Buffer> = Validity<T, Buffer>`
#[derive(Clone, Copy, Debug)]
pub struct Nullable;

impl Nullability for Nullable {
    const NULLABLE: bool = true;

    /// Nullable items are wrapped in an [`Option`].
    type Item<T> = Option<T>;

    /// Nullable collections are wrapped together with a
    /// [`crate::bitmap::Bitmap`].
    type Collection<T: Collection, Buffer: BufferType> = Validity<T, Buffer>;
}
