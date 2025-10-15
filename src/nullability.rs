//! Nullable and non-nullable data.

use std::borrow::Borrow;

use crate::{buffer::Buffer, collection::Collection, validity::Validity};

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
    /// Generic over a collection `T` and a [`Buffer`].
    type Collection<T: Collection, Storage: Buffer>: Collection<
        Owned = Self::Item<<T as Collection>::Owned>,
    >;

    /// Convert to a reference.
    fn as_ref<T>(item: &Self::Item<T>) -> Self::Item<&T>;

    /// Borrow
    fn borrow<T: Borrow<U>, U: ?Sized>(item: &Self::Item<T>) -> Self::Item<&U>;

    /// Maps an item using the provided function.
    fn map<T, U, F: FnOnce(T) -> U>(item: Self::Item<T>, f: F) -> Self::Item<U>;

    /// Maps a reference item using the provided function.
    fn map_ref<T, U, F: FnOnce(&T) -> U>(item: Self::Item<T>, f: F) -> Self::Item<U>;

    /// Zip an item with another item.
    fn zip<T, U>(item: Self::Item<T>, other: Self::Item<U>) -> Self::Item<(T, U)>;

    /// Zip item with other and apply f.
    fn zip_with<T, U, R, F: FnOnce((T, U)) -> R>(
        item: Self::Item<T>,
        other: Self::Item<U>,
        f: F,
    ) -> Self::Item<R> {
        Self::map(Self::zip::<T, U>(item, other), f)
    }
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NonNullable;

impl Nullability for NonNullable {
    const NULLABLE: bool = false;

    /// Non-nullable items are just `T`.
    type Item<T> = T;

    /// Non-nullable collections are just `T`.
    type Collection<T: Collection, Storage: Buffer> = T;

    fn as_ref<T>(item: &Self::Item<T>) -> Self::Item<&T> {
        item
    }

    fn borrow<T: Borrow<U>, U: ?Sized>(item: &Self::Item<T>) -> Self::Item<&U> {
        item.borrow()
    }

    fn map<T, U, F: FnOnce(T) -> U>(item: Self::Item<T>, f: F) -> Self::Item<U> {
        f(item)
    }

    fn map_ref<T, U, F: FnOnce(&T) -> U>(item: Self::Item<T>, f: F) -> Self::Item<U> {
        f(&item)
    }

    fn zip<T, U>(item: Self::Item<T>, other: Self::Item<U>) -> Self::Item<(T, U)> {
        (item, other)
    }
}

/// Nullable types.
///
/// Implements [`Nullability`] to provide:
/// - `Nullable::Item<T> = Option<T>`
/// - `Nullable::Collection<T, Buffer> = Validity<T, Buffer>`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nullable;

impl Nullability for Nullable {
    const NULLABLE: bool = true;

    /// Nullable items are wrapped in an [`Option`].
    type Item<T> = Option<T>;

    /// Nullable collections are wrapped together with a
    /// [`crate::bitmap::Bitmap`].
    type Collection<T: Collection, Storage: Buffer> = Validity<T, Storage>;

    fn as_ref<T>(item: &Self::Item<T>) -> Self::Item<&T> {
        item.as_ref()
    }

    fn borrow<T: Borrow<U>, U: ?Sized>(item: &Self::Item<T>) -> Self::Item<&U> {
        item.as_ref().map(|item| item.borrow())
    }

    fn map<T, U, F: FnOnce(T) -> U>(item: Self::Item<T>, f: F) -> Self::Item<U> {
        item.map(f)
    }

    fn map_ref<T, U, F: FnOnce(&T) -> U>(item: Self::Item<T>, f: F) -> Self::Item<U> {
        item.as_ref().map(f)
    }

    fn zip<T, U>(item: Self::Item<T>, other: Self::Item<U>) -> Self::Item<(T, U)> {
        item.zip(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip() {
        assert_eq!(NonNullable::zip(1, 2), (1, 2));
        assert_eq!(Nullable::zip::<i32, i32>(None, None), None);
        assert_eq!(Nullable::zip::<_, i32>(Some(1), None), None);
        assert_eq!(Nullable::zip::<i32, _>(None, Some(2)), None);
        assert_eq!(Nullable::zip(Some(1), Some(2)), Some((1, 2)));
    }

    #[test]
    fn generic() {
        #[derive(Debug, PartialEq)]
        struct Add<Nulls: Nullability>(Nulls::Item<u8>);

        impl<Nulls: Nullability> Add<Nulls> {
            fn add(self, other: Self) -> Nulls::Item<u8> {
                Nulls::zip_with(self.0, other.0, |(a, b)| a + b)
            }
        }

        assert_eq!(Add::<NonNullable>(1).add(Add(2)), 3);
        assert_eq!(Add::<Nullable>(Some(1)).add(Add(Some(2))), Some(3));
        assert_eq!(Add::<Nullable>(Some(1)).add(Add(None)), None);
    }
}
