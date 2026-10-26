//! Nullable and non-nullable data.

use crate::{
    bitmap::ValidityBitmap,
    buffer::Buffer,
    collection::{ChildRef, Collection},
    validity::Validity,
};

/// Nullability trait for nullable and non-nullable type constructors.
///
/// See [`NonNullable`] and [`Nullable`].
///
/// Nullability is a type constructor independent of the logical layout and
/// storage backend. The same layout can therefore describe required and
/// optional values without duplicating its implementation:
///
/// ```text
/// NonNullable: T         -> values
/// Nullable:    Option<T> -> Validity<values>
/// ```
///
/// # Examples
///
/// ```
/// use narrow::{buffer::VecBuffer, length::Length};
/// use narrow::nullability::{NonNullable, Nullability, Nullable};
///
/// let required: <NonNullable as Nullability>::Item<u8> = 1;
/// let optional: <Nullable as Nullability>::Item<u8> = Some(1);
/// assert_eq!((required, optional), (1, Some(1)));
/// assert!(!NonNullable::NULLABLE && Nullable::NULLABLE);
/// let values: <Nullable as Nullability>::Collection<Vec<u8>, VecBuffer> =
///     [Some(1), None].into_iter().collect();
/// assert_eq!(values.len(), 2);
/// ```
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

    /// Returns the underlying collection without applying validity.
    fn collection_ref<T: Collection, Storage: Buffer>(
        collection: &Self::Collection<T, Storage>,
    ) -> &T;

    /// Projects an in-bounds, valid row without reading other row values.
    ///
    /// The callback receives the raw collection and is called only for a valid
    /// row. `None` means out of bounds (or a missing projected value). Nullable
    /// collections return `Some(None)` for a null row without calling `f`.
    ///
    /// ```
    /// use narrow::{buffer::VecBuffer, nullability::{Nullability, Nullable}, validity::Validity};
    /// let rows = [None::<i32>].into_iter().collect::<Validity<Vec<i32>>>();
    /// let projected = Nullable::project::<_, VecBuffer, i32, _>(&rows, 0, |_| panic!("null row"));
    /// assert_eq!(projected, Some(None));
    /// let missing = Nullable::project::<_, VecBuffer, i32, _>(&rows, 1, |_| panic!("out of bounds"));
    /// assert_eq!(missing, None);
    /// ```
    fn project<'a, T: Collection + 'a, Storage: Buffer, U, F: FnOnce(&'a T) -> Option<U>>(
        collection: &'a Self::Collection<T, Storage>,
        index: usize,
        f: F,
    ) -> Option<Self::Item<U>>;

    /// Maps an item using the provided function.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::nullability::{Nullability, Nullable};
    ///
    /// assert_eq!(Nullable::map(Some(2), |value| value * 2), Some(4));
    /// ```
    fn map<T, U, F: FnOnce(T) -> U>(item: Self::Item<T>, f: F) -> Self::Item<U>;

    /// Zip an item with another item.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::nullability::{Nullability, Nullable};
    ///
    /// assert_eq!(Nullable::zip(Some(1), Some(2)), Some((1, 2)));
    /// assert_eq!(Nullable::zip(Some(1), None::<u8>), None);
    /// ```
    fn zip<T, U>(item: Self::Item<T>, other: Self::Item<U>) -> Self::Item<(T, U)>;

    /// Zip item with other and apply f.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::nullability::{Nullability, NonNullable};
    ///
    /// assert_eq!(NonNullable::zip_with(1, 2, |(a, b)| a + b), 3);
    /// ```
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
    impl Sealed for super::NonNullable {}
    impl Sealed for super::Nullable {}
}

/// Non-nullable types.
///
/// Implements [`Nullability`] to provide:
/// - `NonNullable::Item<T> = T`
/// - `NonNullable::Collection<T, Buffer> = T`
///
/// The identity constructor represents Arrow fields that need no validity
/// bitmap, so required data keeps exactly its underlying physical layout.
///
/// # Examples
///
/// ```
/// use narrow::nullability::{NonNullable, Nullability};
///
/// let value: <NonNullable as Nullability>::Item<u8> = 1;
/// assert_eq!(value, 1);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NonNullable;

impl Nullability for NonNullable {
    const NULLABLE: bool = false;

    /// Non-nullable items are just `T`.
    type Item<T> = T;

    /// Non-nullable collections are just `T`.
    type Collection<T: Collection, Storage: Buffer> = T;

    fn collection_ref<T: Collection, Storage: Buffer>(collection: &T) -> &T {
        collection
    }

    fn project<'a, T: Collection + 'a, Storage: Buffer, U, F: FnOnce(&'a T) -> Option<U>>(
        collection: &'a T,
        index: usize,
        f: F,
    ) -> Option<U> {
        (index < collection.len()).then(|| f(collection)).flatten()
    }

    fn map<T, U, F: FnOnce(T) -> U>(item: Self::Item<T>, f: F) -> Self::Item<U> {
        f(item)
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
///
/// `Option` appears at the item boundary while [`Validity`] keeps Arrow's
/// value and validity buffers separate in the collection representation.
///
/// # Examples
///
/// ```
/// use narrow::nullability::{Nullability, Nullable};
///
/// let value: <Nullable as Nullability>::Item<u8> = None;
/// assert_eq!(value, None);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nullable;

impl Nullability for Nullable {
    const NULLABLE: bool = true;

    /// Nullable items are wrapped in an [`Option`].
    type Item<T> = Option<T>;

    /// Nullable collections are wrapped with optional validity bitmap
    /// storage.
    type Collection<T: Collection, Storage: Buffer> = Validity<T, Storage>;

    fn collection_ref<T: Collection, Storage: Buffer>(collection: &Validity<T, Storage>) -> &T {
        collection.child_ref()
    }

    fn project<'a, T: Collection + 'a, Storage: Buffer, U, F: FnOnce(&'a T) -> Option<U>>(
        collection: &'a Validity<T, Storage>,
        index: usize,
        f: F,
    ) -> Option<Option<U>> {
        if collection.is_valid(index)? {
            f(collection.child_ref()).map(Some)
        } else {
            Some(None)
        }
    }

    fn map<T, U, F: FnOnce(T) -> U>(item: Self::Item<T>, f: F) -> Self::Item<U> {
        item.map(f)
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
