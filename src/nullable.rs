use crate::{
    Bitmap, BitmapIter, Buffer, DataBuffer, Length, Null, ValidityBitmap, DEFAULT_ALIGNMENT,
};
use std::iter::{Map, Zip};

/// Wrapper for nullable data.
///
/// Allocates a validity [Bitmap] that stores a single bit per value in `T`
/// that indicates the nullness or non-nullness of that value.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Nullable<T, const A: usize = DEFAULT_ALIGNMENT> {
    data: T,
    validity: Bitmap<A>,
}

impl<T, const A: usize> Nullable<T, A> {
    /// # Safety
    /// Caller must ensure: todo(mb)
    pub unsafe fn from_raw_parts(data: T, validity: Bitmap<A>) -> Self {
        Self { data, validity }
    }
}

impl<T, const A: usize> Null for Nullable<T, A> {
    unsafe fn is_valid_unchecked(&self, index: usize) -> bool {
        self.validity.is_valid_unchecked(index)
    }

    fn valid_count(&self) -> usize {
        self.validity.valid_count()
    }

    fn null_count(&self) -> usize {
        self.validity.null_count()
    }
}

// todo(mb): replace with buffer traits
// impl<const A: usize, const B: usize> Nullable<Bitmap<A>, B> {
//     pub fn iter_data(&self) -> BitmapIter<'_> {
//         self.data.into_iter()
//     }
// }

// todo(mb): replace with buffer traits
// impl<T, const A: usize, const B: usize> Nullable<Buffer<T, A>, B>
// where
//     T: Copy,
// {
//     pub fn iter_data(&self) -> Iter<'_, T> {
//         self.data.iter()
//     }
// }

// impl<const A: usize, const B: usize> Index<usize> for Nullable<Bitmap<A>, B> {
//     type Output = bool;

//     fn index(&self, index: usize) -> &Self::Output {
//         self.data.index(index)
//     }
// }

// impl<T, const A: usize, const B: usize> Index<usize> for Nullable<Buffer<T, A>, B> {
//     type Output = T;

//     fn index(&self, index: usize) -> &Self::Output {
//         self.data.index(index)
//     }
// }

pub type NullableIter<'a, T> = Map<
    Zip<BitmapIter<'a>, <&'a T as IntoIterator>::IntoIter>,
    fn((bool, <&'a T as IntoIterator>::Item)) -> Option<<&'a T as IntoIterator>::Item>,
>;

impl<'a, T, const A: usize> IntoIterator for &'a Nullable<T, A>
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

impl<T, U, const A: usize> FromIterator<Option<U>> for Nullable<T, A>
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

impl<T, const A: usize> Length for Nullable<T, A> {
    fn len(&self) -> usize {
        self.validity.len()
    }
}

impl<T, const A: usize, const B: usize> DataBuffer<T, A> for Nullable<Buffer<T, A>, B> {
    fn data_buffer(&self) -> &Buffer<T, A> {
        &self.data
    }
}

// todo for bitmap? data?

impl<T, const A: usize> ValidityBitmap<A> for Nullable<T, A> {
    fn validity_bitmap(&self) -> &Bitmap<A> {
        &self.validity
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
            nullable.validity,
            [true, false, true, true].into_iter().collect::<Bitmap>()
        );
        assert_eq!(
            nullable.data,
            [1, u32::default(), 3, 4].into_iter().collect()
        );
        assert_eq!(nullable.len(), 4);
        assert_eq!(nullable.null_count(), 1);
        assert_eq!(nullable.valid_count(), 3);

        let nullable = Vec::<Option<bool>>::new()
            .into_iter()
            .collect::<Nullable<Bitmap>>();
        assert_eq!(nullable.validity, Bitmap::default());
        assert_eq!(nullable.data, Bitmap::default());
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

        let x = Foo { count: 1234 };
        let bitmap: Nullable<Bitmap> = x.into_iter().collect();
        assert_eq!(bitmap.len(), 1234);
    }

    #[test]
    fn into_iter() {
        let nullable: Nullable<Buffer<u8, 1>> =
            [Some(1u8), None, Some(3), Some(4)].into_iter().collect();
        let vec = nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(vec, &[Some(&1u8), None, Some(&3), Some(&4)]);
    }
}
