//! Nullable and non-nullable types.

use crate::{buffer::BufferType, validity::Validity, Index, Length};

pub trait Collection: Index + Length {
    /// The items stored in this collection.
    type Item;

    /// Reference to items stored in this collection.
    type RefItem<'a>
    where
        Self: 'a;

    /// Iterator over ref items.
    type Iter<'a>: Iterator<Item = Self::RefItem<'a>>
    where
        Self: 'a;

    /// Iterator over items in this collection.
    type IntoIter: Iterator<Item = <Self as Collection>::Item>;

    /// Iterate over reference items.
    fn iter(&self) -> Self::Iter<'_>;

    /// Turn collection into iterator.
    fn into_iter(self) -> Self::IntoIter;
}

/// Nullability trait for nullable and non-nullable type constructors
pub trait Nullability: sealed::Sealed {
    /// `true` iff this is [`Nullable`].
    const NULLABLE: bool;

    /// Constructor for nullable and non-nullable items.
    ///
    /// Generic over an item `T`.
    type Item<T>;

    /// Constructor for nullable and non-nullable collections.
    ///
    /// Generic over a collection `T` and a [`BufferType`].
    type Collection<T: Collection<Item = Self::Item<T>>, Buffer: BufferType>: Collection;
}

/// Private module for [`sealed::Sealed`] trait.
mod sealed {
    /// Used to seal [`super::Nullability`].
    pub trait Sealed {}

    /// Prevent downstream implementation of [`super::Nullability`].
    impl<T> Sealed for T where T: super::Nullability {}
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
    type Collection<T: Collection<Item = Self::Item<T>>, Buffer: BufferType> = Validity<T, Buffer>;
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
    type Collection<T: Collection<Item = Self::Item<T>>, Buffer: BufferType> = T;
}
