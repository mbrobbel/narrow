use crate::{Bitmap, Data, Primitive};
use bitvec::{order::Lsb0, slice::BitValIter};
use std::iter::{FromIterator, Map, Zip};

/// Wrapper for nullable Arrow data.
#[derive(Debug)]
pub struct Nullable<T>
where
    T: Data,
{
    validity: Bitmap,
    data: T,
}

impl<T> Nullable<T>
where
    T: Data,
{
    /// Returns an empty [Nullable].
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Buffer, Nullable};
    ///
    /// let empty = Nullable::<Buffer<u8, 0>>::empty();
    ///
    /// assert!(empty.validity().is_empty());
    /// assert!(empty.data().is_empty());
    /// ```
    pub fn empty() -> Self {
        Self {
            validity: Bitmap::empty(),
            data: T::default(),
        }
    }

    /// Returns a reference to the validity [Bitmap] of the [Nullable].
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Bitmap, Buffer, Nullable, ALIGNMENT};
    ///
    /// let nullable: Nullable<Buffer<u32, ALIGNMENT>> =
    ///     [Some(1u32), None, Some(3), Some(4)].into();
    ///
    /// assert_eq!(
    ///     nullable.validity(),
    ///     &Bitmap::from([true, false, true, true])
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
    /// use narrow::{Bitmap, Buffer, Nullable, ALIGNMENT};
    ///
    /// let nullable: Nullable<Buffer<u32, ALIGNMENT>> =
    ///     [Some(1u32), None, Some(3), Some(4)].into();
    ///
    /// assert_eq!(nullable.data(), &Buffer::from([1u32, u32::default(), 3, 4]));
    /// ```
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Constructor for [Nullable]. Used in [Offset](crate::Offset).
    pub(crate) fn new(validity: Bitmap, data: T) -> Self {
        Self { validity, data }
    }
}

impl<T> Data for Nullable<T>
where
    T: Data,
{
    fn len(&self) -> usize {
        self.validity.len()
    }

    fn null_count(&self) -> usize {
        self.validity.count_zeros()
    }

    fn valid_count(&self) -> usize {
        self.validity.count_ones()
    }
}

impl<T> Default for Nullable<T>
where
    T: Data,
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<T, U, const N: usize> From<[Option<U>; N]> for Nullable<T>
where
    T: Data + From<Vec<U>>,
    U: Primitive,
{
    fn from(array: [Option<U>; N]) -> Self {
        array.iter().collect()
    }
}

impl<T, U> From<Box<[Option<U>]>> for Nullable<T>
where
    T: Data + From<Vec<U>>,
    U: Primitive,
{
    fn from(boxed_slice: Box<[Option<U>]>) -> Self {
        boxed_slice.iter().collect()
    }
}

impl<T, U> From<&[Option<U>]> for Nullable<T>
where
    T: Data + From<Vec<U>>,
    U: Primitive,
{
    fn from(slice: &[Option<U>]) -> Self {
        slice.iter().collect()
    }
}

impl<T, U> From<Vec<Option<U>>> for Nullable<T>
where
    T: Data + From<Vec<U>>,
    U: Primitive,
{
    fn from(vec: Vec<Option<U>>) -> Self {
        vec.into_iter().collect()
    }
}

// todo(mb): use unzip with https://github.com/rust-lang/rust/issues/72631
impl<T, U> FromIterator<Option<U>> for Nullable<T>
where
    T: Data + From<Vec<U>>,
    U: Copy + Default,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<U>>,
    {
        let iter = iter.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();
        let mut buffer = Vec::with_capacity(upper_bound.unwrap_or(lower_bound));

        let validity = iter
            .inspect(|opt| {
                buffer.push(opt.unwrap_or_default());
            })
            .map(|opt| opt.is_some())
            .collect::<Bitmap>();

        Self {
            data: buffer.into(),
            validity,
        }
    }
}

impl<'a, T, U> FromIterator<&'a Option<U>> for Nullable<T>
where
    T: Data + From<Vec<U>>,
    U: Copy + Default + 'a,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a Option<U>>,
    {
        iter.into_iter().copied().collect()
    }
}

type NullableIter<'a, T> = Map<
    Zip<BitValIter<'a, Lsb0, usize>, <&'a T as IntoIterator>::IntoIter>,
    fn((bool, <&'a T as IntoIterator>::Item)) -> Option<<&'a T as IntoIterator>::Item>,
>;

impl<'a, T> IntoIterator for &'a Nullable<T>
where
    T: Data,
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
    fn empty() {
        let nullable = Nullable::<Buffer<u8, 0>>::empty();
        assert_eq!(nullable.validity(), &Bitmap::empty());
        assert_eq!(nullable.data(), &Buffer::empty());
        assert_eq!(nullable.len(), 0);
        assert_eq!(nullable.null_count(), 0);
        assert_eq!(nullable.valid_count(), 0);
        assert!(nullable.is_empty());
    }

    #[test]
    fn from_iter() {
        let nullable = vec![Some(1u32), None, Some(3), Some(4)]
            .into_iter()
            .collect::<Nullable<Buffer<_, 3>>>();
        assert_eq!(
            nullable.validity(),
            &Bitmap::from([true, false, true, true])
        );
        assert_eq!(nullable.data(), &Buffer::from([1, u32::default(), 3, 4]));
        assert_eq!(nullable.len(), 4);
        assert_eq!(nullable.null_count(), 1);
        assert_eq!(nullable.valid_count(), 3);
        assert!(!nullable.is_empty());

        let nullable = Vec::<Option<bool>>::new()
            .into_iter()
            .collect::<Nullable<Bitmap>>();
        assert_eq!(nullable.validity(), &Bitmap::empty());
        assert_eq!(nullable.data(), &Bitmap::empty());
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
        let nullable = Nullable::<Buffer<u8, 1>>::from([Some(1u8), None, Some(3), Some(4)]);
        let vec = nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(vec, vec![Some(1u8), None, Some(3), Some(4)]);
    }
}
