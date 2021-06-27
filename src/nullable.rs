use crate::{ArrayData, Bitmap};
use bitvec::{order::Lsb0, slice::BitValIter};
use std::iter::{FromIterator, Map, Zip};

/// Wrapper for nullables data.
///
/// Allocates a validity [Bitmap] that stores a single bit per value in `T`
/// that indicates the nullness or non-nullness of that value.
#[derive(Clone, Debug, Default)]
pub struct Nullable<T> {
    data: T,
    validity: Bitmap,
}

impl<T> Nullable<T> {
    /// Constructor for [Nullable].
    pub(crate) fn new(data: T, validity: Bitmap) -> Self {
        Self { data, validity }
    }

    /// Returns a reference to the validity [Bitmap] of the [Nullable].
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Bitmap, Buffer, Nullable};
    ///
    /// let nullable: Nullable<Buffer<u32, 6>> =
    ///     [Some(1u32), None, Some(3), Some(4)].iter().copied().collect();
    ///
    /// assert_eq!(
    ///     nullable.validity(),
    ///     &[true, false, true, true].iter().copied().collect::<Bitmap>()
    /// );
    /// ```
    pub fn validity(&self) -> &Bitmap {
        &self.validity
    }

    /// Returns a reference to the data of the [Nullable].
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Bitmap, Buffer, Nullable};
    ///
    /// let nullable: Nullable<Buffer<u32, 6>> =
    ///     [Some(1u32), None, Some(3), Some(4)].iter().copied().collect();
    ///
    /// assert_eq!(nullable.data(), &[1u32, u32::default(), 3, 4].iter().copied().collect());
    /// ```
    pub fn data(&self) -> &T {
        &self.data
    }
}

impl<T> ArrayData for Nullable<T> {
    fn len(&self) -> usize {
        self.validity.len()
    }

    fn is_null(&self, index: usize) -> bool {
        self.validity.is_null(index)
    }

    fn null_count(&self) -> usize {
        self.validity.count_zeros()
    }

    fn is_valid(&self, index: usize) -> bool {
        self.validity.is_valid(index)
    }

    fn valid_count(&self) -> usize {
        self.validity.count_ones()
    }
}

impl<T, U> FromIterator<Option<U>> for Nullable<T>
where
    T: FromIterator<U>,
    U: Default,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<U>>,
    {
        let iter = iter.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();
        let mut buffer = Vec::with_capacity(upper_bound.unwrap_or(lower_bound));

        // todo(mb): use unzip with https://github.com/rust-lang/rust/issues/72631
        let validity = iter
            .map(|opt| {
                let validity = opt.is_some();
                buffer.push(opt.unwrap_or_default());
                validity
            })
            .collect();

        Self {
            data: buffer.into_iter().collect(),
            validity,
        }
    }
}

type NullableIter<'a, T> = Map<
    Zip<BitValIter<'a, Lsb0, usize>, <&'a T as IntoIterator>::IntoIter>,
    fn((bool, <&'a T as IntoIterator>::Item)) -> Option<<&'a T as IntoIterator>::Item>,
>;

impl<'a, T> IntoIterator for &'a Nullable<T>
where
    &'a T: IntoIterator,
{
    type Item = Option<<&'a T as IntoIterator>::Item>;
    type IntoIter = NullableIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.validity
            .into_iter()
            .zip(self.data.into_iter())
            .map(|(validity, value)| validity.then(|| value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Buffer;

    #[test]
    fn from_iter() {
        let nullable = vec![Some(1u32), None, Some(3), Some(4)]
            .into_iter()
            .collect::<Nullable<Buffer<_, 3>>>();
        assert_eq!(
            nullable.validity(),
            &[true, false, true, true]
                .iter()
                .copied()
                .collect::<Bitmap>()
        );
        assert_eq!(
            nullable.data(),
            &[1, u32::default(), 3, 4].iter().copied().collect()
        );
        assert_eq!(nullable.len(), 4);
        assert_eq!(nullable.null_count(), 1);
        assert_eq!(nullable.valid_count(), 3);

        let nullable = Vec::<Option<bool>>::new()
            .into_iter()
            .collect::<Nullable<Bitmap>>();
        assert_eq!(nullable.validity(), &Bitmap::default());
        assert_eq!(nullable.data(), &Bitmap::default());
        assert_eq!(nullable.len(), 0);

        struct Foo {
            count: usize,
        }

        impl Iterator for Foo {
            type Item = Option<bool>;

            fn next(&mut self) -> Option<Self::Item> {
                if self.count != 0 {
                    self.count -= 1;
                    Some(Some(true))
                } else {
                    None
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (0, None)
            }
        }

        let foo = Foo { count: 1234 };
        let bitmap = Nullable::<Bitmap>::from_iter(foo);
        assert_eq!(bitmap.len(), 1234);
    }

    #[test]
    fn into_iter() {
        let nullable = Nullable::<Buffer<u8, 1>>::from_iter(
            [Some(1u8), None, Some(3), Some(4)].iter().copied(),
        );
        let vec = nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(vec, vec![Some(1u8), None, Some(3), Some(4)]);
    }
}
