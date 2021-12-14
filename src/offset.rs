use crate::{
    BitmapIter, Buffer, DataBuffer, Length, Null, Nullable, Primitive, Validity, ValidityBitmap,
    DEFAULT_ALIGNMENT,
};
use std::{
    iter::{Map, Skip, Zip},
    num::TryFromIntError,
    ops::{AddAssign, Deref},
    slice::Iter,
};

/// Types representing offset values.
///
/// Values with these types can be used to represent offset values in an
/// [Offset].
///
/// This trait is sealed to prevent downstream implementations.
pub trait OffsetValue:
    Primitive
    + AddAssign
    + TryFrom<usize, Error = TryFromIntError>
    + TryInto<usize, Error = TryFromIntError>
    + sealed::Sealed
{
}

mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T where T: super::OffsetValue {}
}

impl OffsetValue for i32 {}
impl OffsetValue for i64 {}

/// Wrapper for offset values of variable sized arrays.
///
/// This is a wrapper around an inner `T` and a [Validity] with a [Buffer] that
/// provides a different [FromIterator] implementation. In the offset buffer the
/// previous offset is copied for null values, whereas the behavior of
/// [Validity] adds a [Default::default] value.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Offset<T, U, const N: bool, const A: usize = DEFAULT_ALIGNMENT> {
    inner: T,
    offset: Validity<Buffer<U, A>, N>,
}

impl<T, U, const N: bool, const A: usize> Clone for Offset<T, U, N, A>
where
    T: Clone,
    U: OffsetValue,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            offset: self.offset.clone(),
        }
    }
}

// Deref for Null impl of Validity (for valid data)
impl<T, U, const A: usize> Deref for Offset<T, U, false, A> {
    type Target = Validity<Buffer<U, A>, false>;

    fn deref(&self) -> &Self::Target {
        &self.offset
    }
}

impl<T, U, V, const A: usize> FromIterator<V> for Offset<T, U, false, A>
where
    T: FromIterator<V::Item>,
    U: OffsetValue,
    V: IntoIterator + Length, // asref<[]> + extend from slice for buffer/bitmap
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = V>,
    {
        let iter = iter.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();

        let mut buffer = Vec::with_capacity(upper_bound.unwrap_or(lower_bound) + 1);
        buffer.push(U::default());

        let inner = iter
            .scan(U::default(), |last, item| {
                *last += U::try_from(item.len()).unwrap();
                buffer.push(*last);
                Some(item)
            })
            .flat_map(|iter| iter.into_iter())
            .collect();

        Self {
            inner,
            offset: buffer.into_iter().collect(),
        }
    }
}

impl<T, U, V, const A: usize> FromIterator<Option<V>> for Offset<T, U, true, A>
where
    T: FromIterator<V::Item>,
    U: OffsetValue,
    V: IntoIterator + Length, // maybe use asref<[]> here and extend_from_slice for buffer/bitmap
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<V>>,
    {
        let iter = iter.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();

        let mut buffer = Vec::with_capacity(upper_bound.unwrap_or(lower_bound) + 1);
        buffer.push(U::default());

        // todo(mb): replace with something more efficient
        let mut bitmap = Vec::with_capacity(upper_bound.unwrap_or(lower_bound));

        let inner = iter
            .scan(U::default(), |last, opt| {
                bitmap.push(opt.is_some());
                *last += opt
                    .as_ref()
                    .map(|item| U::try_from(item.len()).unwrap())
                    .unwrap_or_default();
                buffer.push(*last);
                Some(opt)
            })
            .flatten()
            .flat_map(|iter| iter.into_iter())
            .collect();

        Self {
            inner,
            offset: unsafe {
                Nullable::from_raw_parts(buffer.into_iter().collect(), bitmap.into_iter().collect())
            }
            .into(),
        }
    }
}

impl<T, U, const A: usize> Length for Offset<T, U, false, A> {
    fn len(&self) -> usize {
        self.offset.len() - 1
    }
}

impl<T, U, const A: usize> Length for Offset<T, U, true, A> {
    fn len(&self) -> usize {
        self.offset.len()
    }
}

// deref is blocked because into_iter is weird when bitmap and buffer are
// different lengths.
impl<T, U, const A: usize> Null for Offset<T, U, true, A> {
    unsafe fn is_valid_unchecked(&self, index: usize) -> bool {
        self.offset.is_valid_unchecked(index)
    }

    fn valid_count(&self) -> usize {
        self.offset.valid_count()
    }

    fn null_count(&self) -> usize {
        self.offset.null_count()
    }
}

type OffsetIter<'a, T> =
    Map<Zip<Iter<'a, T>, Skip<Iter<'a, T>>>, fn((&'a T, &'a T)) -> (usize, usize)>;

impl<'a, T, U, const A: usize> IntoIterator for &'a Offset<T, U, false, A>
where
    U: OffsetValue,
{
    type Item = (usize, usize);
    type IntoIter = OffsetIter<'a, U>;

    fn into_iter(self) -> Self::IntoIter {
        self.offset
            .into_iter()
            .zip(self.offset.into_iter().skip(1))
            .map(|(&start, &end)| (start.try_into().unwrap(), end.try_into().unwrap()))
    }
}

type NullableOffsetIter<'a, T> = Map<
    Zip<Zip<Iter<'a, T>, Skip<Iter<'a, T>>>, BitmapIter<'a>>,
    fn(((&'a T, &'a T), bool)) -> Option<(usize, usize)>,
>;

impl<'a, T, U, const A: usize> IntoIterator for &'a Offset<T, U, true, A>
where
    U: OffsetValue,
{
    type Item = Option<(usize, usize)>;
    type IntoIter = NullableOffsetIter<'a, U>;

    fn into_iter(self) -> Self::IntoIter {
        self.offset
            .data_buffer()
            .into_iter()
            .zip(self.offset.data_buffer().into_iter().skip(1))
            .zip(self.offset.validity_bitmap().into_iter())
            .map(|((&start, &end), validity)| {
                validity.then(|| (start.try_into().unwrap(), end.try_into().unwrap()))
            })
    }
}

// pub struct OffsetsIter<'a, T> {
//     pos: usize,
//     offsets: T,
// }

// impl<'a, T> Iterator for OffsetsIter<'a, Iter<'a, T>>
// where
//     T: OffsetValue<'a>,
// {
//     type Item = (usize, usize);

//     fn next(&mut self) -> Option<Self::Item> {
//         self.offsets
//             .next()
//             .map(|offset| offset.try_into().unwrap())
//             .map(|end| {
//                 let result = (self.pos, end);
//                 self.pos = end;
//                 result
//             })
//     }
// }

// impl<'a, T> Iterator for OffsetsIter<'a, NullableIter<'a, T>>
// where
//     T: OffsetValue<'a>,
// {
//     type Item = Option<(usize, usize)>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.offsets
//             .next()
//             .map(|(validity, offset)| (validity, offset.try_into().unwrap()))
//             .map(|(validity, end)| {
//                 let result = (self.pos, end);
//                 self.pos = end;
//                 validity.then(|| result)
//             })
//     }
// }

// pub struct OffsetSliceIter<'a, T, U> {
//     data: &'a [T],
//     offsets: U,
// }

// impl<'a, T, U> Iterator for OffsetSliceIter<'a, T, U>
// where
//     U: Iterator<Item = (usize, usize)>,
// {
//     type Item = &'a [T];

//     fn next(&mut self) -> Option<Self::Item> {
//         self.offsets
//             .next()
//             .map(|(start, end)| &self.data[start..end])
//     }
// }

// impl<'a, T, U, const A: usize, const B: usize> IntoIterator
//     for &'a Offset<Buffer<T, A>, U, false, B>
// {
//     type Item = &'a [T];
//     type IntoIter;

//     fn into_iter(self) -> Self::IntoIter {
//         todo!()
//     }
// }

// two types of iterator for offsets:
// - over slices, used for flat variable size binary and utf8 arrays
// - over iterators, used for nested lists?

// fn a<'a, T, U>(offset: &'a Offset<T, U, false>) {
//     let offsets: std::slice::Iter<'a, U> = offset.into_iter();
// }

// pub struct OffsetSliceIter<T, U, V, const N: bool>
// where
//     T: AsRef<[U]>,
// {
//     inner: T,
//     offsets: V,
// }

// pub struct OffsetIter<T, U, const N: bool> {
//     inner: T,
//     offsets:
// }

// /// Iterator over offsets values of variable sized arrays.
// pub struct OffsetIter<T, const N: bool> {
//     pos: Option<usize>,
//     iter: T,
// }

// impl<'a, T> Iterator for OffsetIter<Copied<Iter<'a, T>>, false>
// where
//     T: OffsetValue,
// {
//     type Item = (usize, usize);

//     fn next(&mut self) -> Option<Self::Item> {
//         // Empty offset array?
//         self.pos.and(
//             self.iter
//                 .next()
//                 .map(|offset| offset.try_into().unwrap())
//                 .map(|end| {
//                     let result = (self.pos.unwrap(), end);
//                     self.pos = Some(end);
//                     result
//                 }),
//         )
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.iter.size_hint()
//     }
// }

// impl<'a, T> Iterator for OffsetIter<Zip<BitmapIter<'a>, Copied<Iter<'a, T>>>, true>
// where
//     T: OffsetValue,
// {
//     type Item = Option<(usize, usize)>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.pos.and(
//             self.iter
//                 .next()
//                 .map(|(validity, offset)| (validity, offset.try_into().unwrap()))
//                 .map(|(validity, end)| {
//                     let result = (self.pos.unwrap(), end);
//                     self.pos = Some(end);
//                     match validity {
//                         true => Some(result),
//                         false => None,
//                     }
//                 }),
//         )
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.iter.size_hint()
//     }
// }

// impl<'a, T> IntoIterator for &'a Offset<T, false>
// where
//     T: OffsetValue,
// {
//     type Item = (usize, usize);
//     type IntoIter = OffsetIter<Copied<Iter<'a, T>>, false>;

//     fn into_iter(self) -> Self::IntoIter {
//         let mut iter = self.0.into_iter();
//         OffsetIter {
//             pos: iter.next().map(|offset| offset.try_into().unwrap()),
//             iter,
//         }
//     }
// }

// impl<'a, T> IntoIterator for &'a Offset<T, true>
// where
//     T: OffsetValue,
// {
//     type Item = Option<(usize, usize)>;
//     type IntoIter = OffsetIter<Zip<BitmapIter<'a>, Copied<Iter<'a, T>>>, true>;

//     fn into_iter(self) -> Self::IntoIter {
//         let validity = self.0.validity().into_iter();
//         let mut offsets = self.0.data().into_iter();
//         let pos = offsets.next().map(|offset| offset.try_into().unwrap());
//         let iter = validity.zip(offsets);
//         OffsetIter { pos, iter }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Null, Uint8Array};

    // #[test]
    // fn array_data() {
    //     let offset: Offset<i64, false> = [1, 2, 3, 4].into_iter().collect();
    //     assert_eq!(offset.len(), 5); // 5 is correct here because the offset stores an additional value
    //     assert_eq!(offset.is_null(0), Some(false));
    //     assert_eq!(offset.null_count(), 0);
    //     assert_eq!(offset.is_valid(0), Some(true));
    //     assert_eq!(offset.valid_count(), 5);

    //     let offset: Offset<i32, true> = [Some(3), None, Some(4), Some(0)].into_iter().collect();
    //     assert_eq!(offset.len(), 5);
    //     assert_eq!(offset.is_null(0), Some(false));
    //     assert_eq!(offset.is_null(1), Some(true));
    //     assert_eq!(offset.is_null(2), Some(false));
    //     assert_eq!(offset.is_null(3), Some(false));
    //     assert_eq!(offset.is_null(4), None);
    //     assert_eq!(offset.null_count(), 1);
    //     assert_eq!(offset.is_valid(0), Some(true));
    //     assert_eq!(offset.is_valid(1), Some(false));
    //     assert_eq!(offset.valid_count(), 3);
    // }

    #[test]
    fn from_iter() {
        let offset: Offset<Uint8Array<false>, i64, false> =
            [vec![1u8], vec![2, 3], vec![3, 4, 5], vec![4, 5, 6, 7]]
                .into_iter()
                .collect();
        assert_eq!(&offset.offset[..], &[0, 1, 3, 6, 10]);
        assert_eq!(offset.len(), 4);

        let offset: Offset<Uint8Array<true>, i32, false> = [
            vec![Some(1u8)],
            vec![Some(2), None],
            vec![Some(3), None, None],
            vec![Some(4), None, None, None],
        ]
        .into_iter()
        .collect();
        assert_eq!(&offset.offset[..], &[0, 1, 3, 6, 10]);
        assert_eq!(offset.is_valid(0), Some(true));
        assert_eq!(offset.len(), 4);

        let offset: Offset<Uint8Array<true>, i32, true> = [
            Some(vec![Some(1u8)]),
            None,
            None,
            Some(vec![Some(2), None]),
            Some(vec![Some(3), None, None]),
            Some(vec![Some(4), None, None, None]),
        ]
        .into_iter()
        .collect();
        // todo(mb): this is not what we want, we want to override the iterator impl in offset
        // we need to prevent the into iterator impl from the deref target (by default)
        // this is the motivation to make those apis private.
        // or maybe remove the deref impl for nullable variable width?
        // assert_eq!(
        //     &offset.offset.into_iter().collect::<Vec<_>>(),
        //     &[
        //         Some(&0),
        //         Some(&1),
        //         Some(&1),
        //         Some(&1),
        //         Some(&3),
        //         Some(&6),
        //         Some(&10)
        //     ]
        // );
        // but this can't be used because offset is private field and I removed the deref impl.
        assert_eq!(offset.is_valid(0), Some(true));
        assert_eq!(offset.is_null(1), Some(true));
        assert_eq!(offset.is_null(2), Some(true));
        assert_eq!(offset.is_valid(3), Some(true));
        assert_eq!(offset.len(), 6);

        // let offset: Offset<i32, true> = [Some(3), None, Some(4), Some(0)].into_iter().collect();
        // assert_eq!(&offset.offset[..], &[0, 3, 3, 7, 7]);
    }

    #[test]
    fn into_iter() {
        let offset: Offset<Buffer<u8>, i64, false> =
            [vec![1], vec![2, 3], vec![3, 4, 5], vec![4, 5, 6, 7]]
                .into_iter()
                .collect();
        let offsets = offset.into_iter().collect::<Vec<_>>();
        assert_eq!(offsets, vec![(0, 1), (1, 3), (3, 6), (6, 10)]);

        let offset: Offset<Buffer<u8>, i32, true> = [
            Some(vec![1, 2, 3]),
            None,
            Some(vec![1, 2, 3, 4]),
            None,
            Some(vec![]),
        ]
        .into_iter()
        .collect();
        let offsets = offset.into_iter().collect::<Vec<_>>();
        assert_eq!(
            offsets,
            vec![Some((0, 3)), None, Some((3, 7)), None, Some((7, 7))]
        );
    }
}
