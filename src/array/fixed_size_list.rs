use crate::{Array, ArrayType, Bitmap, NestedArray, Validity};
use std::{
    array::{self, IntoIter},
    iter::{FromIterator, Skip, Take},
};

/// Array with fixed-size lists of other array types.
#[derive(Debug)]
pub struct FixedSizeListArray<T, const N: usize, const M: bool>(Validity<T, M>)
where
    T: Array;

impl<T, const N: usize, const M: bool> Array for FixedSizeListArray<T, N, M>
where
    T: Array,
{
    type Validity = Validity<T, M>;

    fn validity(&self) -> &Self::Validity {
        &self.0
    }

    fn len(&self) -> usize {
        match M {
            // Non-nullable arrays len implementation comes from buffer which
            // is the fixed-size length multiplied by the array length.
            false => crate::ArrayData::len(&self.0) / N,
            true => crate::ArrayData::len(&self.0),
        }
    }
}

impl<T, const N: usize, const M: bool> NestedArray for FixedSizeListArray<T, N, M>
where
    T: Array,
{
    type Child = T;

    fn child(&self) -> &T {
        self.0.data()
    }
}

impl<T, const N: usize> ArrayType for [T; N]
where
    T: Array,
{
    type Array = FixedSizeListArray<T, N, false>;
}

impl<T, const N: usize> ArrayType for Option<[T; N]>
where
    T: Array,
{
    type Array = FixedSizeListArray<T, N, true>;
}

impl<T, U, const N: usize> FromIterator<[U; N]> for FixedSizeListArray<T, N, false>
where
    T: Array + FromIterator<U>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = [U; N]>,
    {
        Self(iter.into_iter().map(IntoIter::new).flatten().collect())
    }
}

impl<T, U, const N: usize> FromIterator<Option<[U; N]>> for FixedSizeListArray<T, N, true>
where
    T: Array + FromIterator<U>,
    U: Default,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<[U; N]>>,
    {
        let iter = iter.into_iter();
        let (_, upper_bound) = iter.size_hint();
        let capacity = upper_bound.expect("iterator has no known upper bound");

        let mut data = Vec::with_capacity(capacity * N);

        let validity = iter
            .map(|opt| match opt {
                Some(value) => {
                    for x in array::IntoIter::new(value) {
                        data.push(x);
                    }
                    true
                }
                None => {
                    for _ in 0..N {
                        data.push(U::default());
                    }
                    false
                }
            })
            .collect::<Bitmap>();

        Self(Validity::nullable(validity, data.into_iter().collect()))
    }
}

/// Iterator over elements of an array with fixed-sized lists of other arrays.
pub struct FixedSizeListArrayIter<'a, T, const N: usize, const M: bool>
where
    T: Array,
{
    position: usize,
    data: &'a FixedSizeListArray<T, N, M>,
}

impl<'a, T, const N: usize> Iterator for FixedSizeListArrayIter<'a, T, N, false>
where
    T: Array,
    &'a T: IntoIterator,
{
    type Item = Take<Skip<<&'a T as IntoIterator>::IntoIter>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.data.len() {
            let result = self.data.0.into_iter().skip(self.position * N).take(N);
            self.position += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a FixedSizeListArray<T, N, false>
where
    T: Array,
    &'a T: IntoIterator,
{
    type Item = Take<Skip<<&'a T as IntoIterator>::IntoIter>>;
    type IntoIter = FixedSizeListArrayIter<'a, T, N, false>;

    fn into_iter(self) -> Self::IntoIter {
        FixedSizeListArrayIter {
            position: 0,
            data: self,
        }
    }
}

impl<'a, T, const N: usize> Iterator for FixedSizeListArrayIter<'a, T, N, true>
where
    T: Array,
    &'a T: IntoIterator,
{
    type Item = Option<Take<Skip<<&'a T as IntoIterator>::IntoIter>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.data.len() {
            let result = if self.data.is_valid(self.position) {
                Some(
                    self.data
                        .0
                        .data()
                        .into_iter()
                        .skip(self.position * N)
                        .take(N),
                )
            } else {
                None
            };
            self.position += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a FixedSizeListArray<T, N, true>
where
    T: Array,
    &'a T: IntoIterator,
{
    type Item = Option<Take<Skip<<&'a T as IntoIterator>::IntoIter>>>;
    type IntoIter = FixedSizeListArrayIter<'a, T, N, true>;

    fn into_iter(self) -> Self::IntoIter {
        FixedSizeListArrayIter {
            position: 0,
            data: self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Uint8Array;

    #[test]
    fn from_iter() {
        let vec = vec![[1u8, 2, 3, 4], [5, 6, 7, 8]];
        let list: FixedSizeListArray<Uint8Array<false>, 4, false> = vec.into_iter().collect();
        assert_eq!(list.len(), 2);
        assert_eq!(list.child().len(), 8);

        let vec = vec![Some([1, 2, 3, 4]), None, None, Some([5, 6, 7, 8]), None];
        let list: FixedSizeListArray<Uint8Array<false>, 4, true> = vec.into_iter().collect();
        assert_eq!(list.len(), 5);
        assert_eq!(list.null_count(), 3);
        assert_eq!(list.valid_count(), 2);
        assert_eq!(list.child().len(), list.len() * 4);
        assert_eq!(list.child().null_count(), 0);
        assert_eq!(list.child().valid_count(), 20);

        let vec = vec![
            Some([Some(1u8), None, Some(3), Some(4)]),
            None,
            None,
            Some([Some(5), None, Some(7), Some(8)]),
        ];
        let list: FixedSizeListArray<Uint8Array<true>, 4, true> = vec.into_iter().collect();
        assert_eq!(list.len(), 4);
        assert_eq!(list.null_count(), 2);
        assert_eq!(list.valid_count(), 2);
        assert_eq!(list.child().len(), 16);
        assert_eq!(list.child().null_count(), 10);
        assert_eq!(list.child().valid_count(), 6);
    }

    #[test]
    fn into_iter() {
        let vec = vec![[1u8, 2, 3, 4], [5, 6, 7, 8]];
        let list: FixedSizeListArray<Uint8Array<false>, 4, false> = vec.iter().copied().collect();
        assert_eq!(
            list.into_iter()
                .map(Iterator::collect)
                .collect::<Vec<Vec<_>>>(),
            vec
        );

        let vec = vec![Some([1, 2, 3, 4]), None, None, Some([5, 6, 7, 8]), None];
        let list: FixedSizeListArray<Uint8Array<false>, 4, true> = vec.iter().copied().collect();
        let mut iter = list.into_iter();
        assert_eq!(
            iter.next().unwrap().map(Iterator::collect),
            Some(vec![1, 2, 3, 4])
        );
        assert!(iter.next().unwrap().is_none());
        assert!(iter.next().unwrap().is_none());
        assert_eq!(
            iter.next().unwrap().map(Iterator::collect),
            Some(vec![5, 6, 7, 8])
        );
        assert!(iter.next().unwrap().is_none());
        assert!(iter.next().is_none());

        let vec = vec![
            Some([Some(1u8), None, Some(3), Some(4)]),
            None,
            None,
            Some([Some(5), None, Some(7), Some(8)]),
        ];
        let list: FixedSizeListArray<Uint8Array<true>, 4, true> = vec.into_iter().collect();
        let mut iter = list.into_iter();
        assert_eq!(
            iter.next().unwrap().map(Iterator::collect),
            Some(vec![Some(1u8), None, Some(3), Some(4)]),
        );
        assert!(iter.next().unwrap().is_none());
        assert!(iter.next().unwrap().is_none());
        assert_eq!(
            iter.next().unwrap().map(Iterator::collect),
            Some(vec![Some(5), None, Some(7), Some(8)]),
        );
        assert!(iter.next().is_none());

        let vec = vec![
            [[1u8, 2, 3, 4], [5, 6, 7, 8]],
            [[1u8, 2, 3, 4], [5, 6, 7, 8]],
        ];
        let list: FixedSizeListArray<FixedSizeListArray<Uint8Array<false>, 4, false>, 2, false> =
            vec.iter().copied().collect();
        assert_eq!(
            list.into_iter()
                .map(|iter| iter.map(Iterator::collect))
                .map(Iterator::collect)
                .collect::<Vec<Vec<Vec<_>>>>(),
            vec
        );
    }
}
