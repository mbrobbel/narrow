//! Offsets for variable sized arrays.

// use crate::{
//     BitmapIter, Buffer, DataBuffer, Length, Null, Nullable, OffsetBuffer, Primitive, Validity,
//     ValidityBitmap,
// };
use crate::Primitive;
use std::{
    // iter::{Map, Skip, Zip},
    num::TryFromIntError,
    ops::AddAssign,
    // slice::Iter,
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

// /// Wrapper for offset values of variable sized arrays.
// ///
// /// This is a wrapper around an inner `T` and a [Validity] with a [Buffer] that
// /// provides a different [FromIterator] implementation. In the offset buffer the
// /// previous offset is copied for null values, whereas the behavior of
// /// [Validity] adds a [Default::default] value.
// #[derive(Debug, PartialEq, Eq, Hash)]
// pub struct Offset<T, U, const N: bool, const A: usize = DEFAULT_ALIGNMENT> {
//     inner: T,
//     offset: Validity<Buffer<U, A>, N>,
// }

// impl<'a, T, U, const N: bool, const A: usize, const B: usize> Offset<Buffer<T, B>, U, N, A>
// where
//     U: OffsetValue,
//     &'a Self: IntoIterator + 'a,
// {
//     pub fn iter_slice(&'a self) -> OffsetSliceIter<'a, T, <&'a Self as IntoIterator>::IntoIter, N> {
//         OffsetSliceIter {
//             data: &self.inner,
//             offset: self.into_iter(),
//         }
//     }
// }

// impl<T, U, const N: bool, const A: usize> Clone for Offset<T, U, N, A>
// where
//     T: Clone,
//     U: OffsetValue,
// {
//     fn clone(&self) -> Self {
//         Self {
//             inner: self.inner.clone(),
//             offset: self.offset.clone(),
//         }
//     }
// }

// impl<T, U, const N: bool, const A: usize, const B: usize> DataBuffer<T, B>
//     for Offset<Buffer<T, B>, U, N, A>
// {
//     fn data_buffer(&self) -> &Buffer<T, B> {
//         &self.inner
//     }
// }

// // // Deref for Null impl of Validity (for valid data)
// // impl<T, U, const A: usize> Deref for Offset<T, U, false, A> {
// //     type Target = Validity<Buffer<U, A>, false>;

// //     fn deref(&self) -> &Self::Target {
// //         &self.offset
// //     }
// // }

// impl<T, U, V, const A: usize> FromIterator<V> for Offset<T, U, false, A>
// where
//     T: FromIterator<V::Item>,
//     U: OffsetValue,
//     V: IntoIterator + Length, // asref<[]> + extend from slice for buffer/bitmap
// {
//     fn from_iter<I>(iter: I) -> Self
//     where
//         I: IntoIterator<Item = V>,
//     {
//         let iter = iter.into_iter();
//         let (lower_bound, upper_bound) = iter.size_hint();

//         let mut buffer = Vec::with_capacity(upper_bound.unwrap_or(lower_bound) + 1);
//         buffer.push(U::default());

//         let inner = iter
//             .scan(U::default(), |last, item| {
//                 *last += U::try_from(item.len()).unwrap();
//                 buffer.push(*last);
//                 Some(item)
//             })
//             .flat_map(|iter| iter.into_iter())
//             .collect();

//         Self {
//             inner,
//             offset: buffer.into_iter().collect(),
//         }
//     }
// }

// impl<T, U, V, const A: usize> FromIterator<Option<V>> for Offset<T, U, true, A>
// where
//     T: FromIterator<V::Item>,
//     U: OffsetValue,
//     V: IntoIterator + Length, // maybe use asref<[]> here and extend_from_slice for buffer/bitmap
// {
//     fn from_iter<I>(iter: I) -> Self
//     where
//         I: IntoIterator<Item = Option<V>>,
//     {
//         let iter = iter.into_iter();
//         let (lower_bound, upper_bound) = iter.size_hint();

//         let mut buffer = Vec::with_capacity(upper_bound.unwrap_or(lower_bound) + 1);
//         buffer.push(U::default());

//         // todo(mb): replace with something more efficient
//         let mut bitmap = Vec::with_capacity(upper_bound.unwrap_or(lower_bound));

//         let inner = iter
//             .scan(U::default(), |last, opt| {
//                 bitmap.push(opt.is_some());
//                 *last += opt
//                     .as_ref()
//                     .map(|item| U::try_from(item.len()).unwrap())
//                     .unwrap_or_default();
//                 buffer.push(*last);
//                 Some(opt)
//             })
//             .flatten()
//             .flat_map(|iter| iter.into_iter())
//             .collect();

//         Self {
//             inner,
//             offset: unsafe {
//                 Nullable::from_raw_parts(buffer.into_iter().collect(), bitmap.into_iter().collect())
//             }
//             .into(),
//         }
//     }
// }

// impl<T, U, const A: usize> Length for Offset<T, U, false, A> {
//     fn len(&self) -> usize {
//         self.offset.len() - 1
//     }
// }

// impl<T, U, const A: usize> Length for Offset<T, U, true, A> {
//     fn len(&self) -> usize {
//         self.offset.len()
//     }
// }

// impl<T, U, const N: bool, const A: usize> Null for Offset<T, U, N, A>
// where
//     Self: Length,
//     Validity<Buffer<U, A>, N>: Null,
// {
//     unsafe fn is_valid_unchecked(&self, index: usize) -> bool {
//         self.offset.is_valid_unchecked(index)
//     }
// }

// // // deref is blocked because into_iter is weird when bitmap and buffer are
// // // different lengths.
// // impl<T, U, const A: usize> Null for Offset<T, U, true, A> {
// //     unsafe fn is_valid_unchecked(&self, index: usize) -> bool {
// //         self.offset.is_valid_unchecked(index)
// //     }

// //     fn valid_count(&self) -> usize {
// //         self.offset.valid_count()
// //     }

// //     fn null_count(&self) -> usize {
// //         self.offset.null_count()
// //     }
// // }

// impl<T, U, const N: bool, const A: usize> OffsetBuffer<U, A> for Offset<T, U, N, A> {
//     fn offset_buffer(&self) -> &Buffer<U, A> {
//         self.offset.data_buffer()
//     }
// }

// pub type OffsetIter<'a, T> =
//     Map<Zip<Iter<'a, T>, Skip<Iter<'a, T>>>, fn((&'a T, &'a T)) -> (usize, usize)>;

// impl<'a, T, U, const A: usize> IntoIterator for &'a Offset<T, U, false, A>
// where
//     U: OffsetValue,
// {
//     type Item = (usize, usize);
//     type IntoIter = OffsetIter<'a, U>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.offset
//             .into_iter()
//             .zip(self.offset.into_iter().skip(1))
//             .map(|(&start, &end)| (start.try_into().unwrap(), end.try_into().unwrap()))
//     }
// }

// pub type NullableOffsetIter<'a, T> = Map<
//     Zip<Zip<Iter<'a, T>, Skip<Iter<'a, T>>>, BitmapIter<'a>>,
//     fn(((&'a T, &'a T), bool)) -> Option<(usize, usize)>,
// >;

// impl<'a, T, U, const A: usize> IntoIterator for &'a Offset<T, U, true, A>
// where
//     U: OffsetValue,
// {
//     type Item = Option<(usize, usize)>;
//     type IntoIter = NullableOffsetIter<'a, U>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.offset
//             .data_buffer()
//             .into_iter()
//             .zip(self.offset.data_buffer().into_iter().skip(1))
//             .zip(self.offset.validity_bitmap().into_iter())
//             .map(|((&start, &end), validity)| {
//                 validity.then(|| (start.try_into().unwrap(), end.try_into().unwrap()))
//             })
//     }
// }

// pub struct OffsetSliceIter<'a, T, U, const N: bool> {
//     data: &'a [T],
//     offset: U,
// }

// impl<'a, T, U> Iterator for OffsetSliceIter<'a, T, U, false>
// where
//     U: Iterator<Item = (usize, usize)>,
// {
//     type Item = &'a [T];

//     fn next(&mut self) -> Option<Self::Item> {
//         self.offset
//             .next()
//             .map(|(start, end)| &self.data[start..end])
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.offset.size_hint()
//     }
// }

// impl<'a, T, U> Iterator for OffsetSliceIter<'a, T, U, true>
// where
//     U: Iterator<Item = Option<(usize, usize)>>,
// {
//     type Item = Option<&'a [T]>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.offset
//             .next()
//             .map(|opt| opt.map(|(start, end)| &self.data[start..end]))
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.offset.size_hint()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{Null, Uint8Array};

//     #[test]
//     fn from_iter() {
//         let offset: Offset<Uint8Array<false>, i64, false> =
//             [vec![1u8], vec![2, 3], vec![3, 4, 5], vec![4, 5, 6, 7]]
//                 .into_iter()
//                 .collect();
//         assert_eq!(&offset.offset[..], &[0, 1, 3, 6, 10]);
//         assert_eq!(offset.len(), 4);

//         let offset: Offset<Uint8Array<true>, i32, false> = [
//             vec![Some(1u8)],
//             vec![Some(2), None],
//             vec![Some(3), None, None],
//             vec![Some(4), None, None, None],
//         ]
//         .into_iter()
//         .collect();
//         assert_eq!(&offset.offset[..], &[0, 1, 3, 6, 10]);
//         assert_eq!(offset.is_valid(0), Some(true));
//         assert_eq!(offset.len(), 4);

//         let offset: Offset<Uint8Array<true>, i32, true> = [
//             Some(vec![Some(1u8)]),
//             None,
//             None,
//             Some(vec![Some(2), None]),
//             Some(vec![Some(3), None, None]),
//             Some(vec![Some(4), None, None, None]),
//         ]
//         .into_iter()
//         .collect();
//         assert_eq!(offset.is_valid(0), Some(true));
//         assert_eq!(offset.is_null(1), Some(true));
//         assert_eq!(offset.is_null(2), Some(true));
//         assert_eq!(offset.is_valid(3), Some(true));
//         assert_eq!(offset.len(), 6);
//     }

//     #[test]
//     fn into_iter() {
//         let offset: Offset<Buffer<u8>, i64, false> =
//             [vec![1], vec![2, 3], vec![3, 4, 5], vec![4, 5, 6, 7]]
//                 .into_iter()
//                 .collect();
//         let offsets = offset.into_iter().collect::<Vec<_>>();
//         assert_eq!(offsets, vec![(0, 1), (1, 3), (3, 6), (6, 10)]);

//         let offset: Offset<Buffer<u8>, i32, true> = [
//             Some(vec![1, 2, 3]),
//             None,
//             Some(vec![1, 2, 3, 4]),
//             None,
//             Some(vec![]),
//         ]
//         .into_iter()
//         .collect();
//         let offsets = offset.into_iter().collect::<Vec<_>>();
//         assert_eq!(
//             offsets,
//             vec![Some((0, 3)), None, Some((3, 7)), None, Some((7, 7))]
//         );
//     }

//     #[test]
//     fn iter_slice() {
//         let offset: Offset<Buffer<u8>, i64, false> =
//             [vec![1], vec![2, 3], vec![3, 4, 5], vec![4, 5, 6, 7]]
//                 .into_iter()
//                 .collect();
//         let mut iter = offset.iter_slice();
//         assert_eq!(iter.next(), Some([1u8].as_slice()));
//         assert_eq!(iter.next(), Some([2u8, 3].as_slice()));
//         assert_eq!(iter.next(), Some([3u8, 4, 5].as_slice()));
//         assert_eq!(iter.next(), Some([4u8, 5, 6, 7].as_slice()));
//         assert_eq!(iter.next(), None);

//         let offset: Offset<Buffer<u8>, i32, true> = [
//             Some(vec![1, 2, 3]),
//             None,
//             Some(vec![1, 2, 3, 4]),
//             None,
//             Some(vec![]),
//         ]
//         .into_iter()
//         .collect();
//         let mut iter = offset.iter_slice();
//         assert_eq!(iter.next(), Some(Some([1u8, 2, 3].as_slice())));
//         assert_eq!(iter.next(), Some(None));
//         assert_eq!(iter.next(), Some(Some([1u8, 2, 3, 4].as_slice())));
//         assert_eq!(iter.next(), Some(None));
//         assert_eq!(iter.next(), Some(Some([].as_slice())));
//         assert_eq!(iter.next(), None);
//         assert_eq!(
//             offset
//                 .iter_slice()
//                 .collect::<Offset<Buffer<u8>, i32, true>>(),
//             offset
//         );
//     }
// }
