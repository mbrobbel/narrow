use crate::{Array, ArrayType, Buffer, Offset, OffsetValue, DEFAULT_ALIGNMENT};
use std::ops::Deref;

/// Array with variable-sized binary data.
#[derive(Debug)]
pub struct VariableSizeBinaryArray<T, const N: bool, const A: usize = DEFAULT_ALIGNMENT>(
    Offset<Buffer<u8, A>, T, N>,
);

impl<T, const N: bool, const A: usize> Array for VariableSizeBinaryArray<T, N, A>
where
    T: OffsetValue,
{
    type Item<'a> = &'a [u8];
}

impl ArrayType for &[u8] {
    type Item<'a> = &'a [u8];
    type Array<T, const N: bool, const A: usize> = VariableSizeBinaryArray<T, false, A>;
}

impl<T, const N: bool, const A: usize> Deref for VariableSizeBinaryArray<T, N, A> {
    type Target = Offset<Buffer<u8, A>, T, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, U, const N: bool, const A: usize> FromIterator<U> for VariableSizeBinaryArray<T, N, A>
where
    Offset<Buffer<u8, A>, T, N>: FromIterator<U>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = U>,
    {
        Self(iter.into_iter().collect())
    }
}
/// Iterator over elements of an array with variable-sized binary data.
pub struct VariableSizeBinaryArrayIter<'a, T, const N: bool>
//, const N: bool, const A: usize>
// where
//     T: OffsetValue,
//     &'a Offset<Buffer<u8, A>, T, N>: IntoIterator,
{
    data: &'a [u8],
    offset: T, //<&'a Offset<Buffer<u8, A>, T, N> as IntoIterator>::IntoIter,
}

impl<'a, T> Iterator for VariableSizeBinaryArrayIter<'a, T, false>
where
    T: Iterator<Item = (usize, usize)>, // T: OffsetValue,
                                        // &'a Offset<Buffer<u8, A>, T, false>: IntoIterator
{
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.offset
            .next()
            .map(|(start, end)| &self.data[start..end])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.offset.size_hint()
    }

    // todo(mb): impl nth and advance_by
}

impl<'a, T, const A: usize> IntoIterator for &'a VariableSizeBinaryArray<T, false, A>
where
    &'a Offset<Buffer<u8, A>, T, false>: IntoIterator,
{
    type Item = &'a [u8];
    type IntoIter = VariableSizeBinaryArrayIter<
        'a,
        <&'a Offset<Buffer<u8, A>, T, false> as IntoIterator>::IntoIter,
        false,
    >;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            data: self.data,
            offset: self.offset.into_iter()
        }
        self.0.into_iter()
    }
}

// impl<'a, T, const N: bool, const A: usize> IntoIterator for &'a VariableSizeBinaryArray<T, N, A>
// where
//     &'a Offset<Buffer<u8, A>, T, N>: IntoIterator,
// {
//     type Item = <&'a Offset<Buffer<u8, A>, T, N> as IntoIterator>::Item;
//     type IntoIter = <&'a Offset<Buffer<u8, A>, T, N> as IntoIterator>::IntoIter;

//     fn into_iter(self) -> Self::IntoIter {
//         self.0.into_iter()
//     }
// }

// todo(mb): use traits to access buffers
// impl<T, const N: bool> VariableSizeBinaryArray<T, N>
// where
//     T: OffsetValue,
// {
//     pub fn data(&self) -> &Buffer<u8, ALIGN> {
//         &self.data
//     }
// }

// impl<T> ArrayIndex<usize> for VariableSizeBinaryArray<T, false>
// where
//     T: OffsetValue,
// {
//     type Output = Vec<u8>;

//     fn index(&self, index: usize) -> Self::Output {
//         let start = (*self.offset.index(index)).try_into().unwrap();
//         let end = (*self.offset.index(index + 1)).try_into().unwrap();
//         self.data[start..end].to_vec()
//     }
// }

// impl<T, U> FromIterator<Option<U>> for VariableSizeBinaryArray<T, true>
// where
//     T: OffsetValue,
//     U: AsRef<[u8]>,
// {
//     fn from_iter<I>(iter: I) -> Self
//     where
//         I: IntoIterator<Item = Option<U>>,
//     {
//         let mut data = Vec::default();

//         let offset = iter
//             .into_iter()
//             .inspect(|opt| {
//                 if let Some(slice) = opt {
//                     data.extend_from_slice(slice.as_ref());
//                 }
//             })
//             .map(|opt| opt.map(|slice| T::try_from(slice.as_ref().len()).unwrap()))
//             .collect();

//         Self {
//             data: data.into_iter().collect(),
//             offset,
//         }
//     }
// }

// impl<T> Index<usize> for VariableSizeBinaryArray<T, false>
// where
//     T: OffsetValue,
// {
//     type Output = [u8];

//     fn index(&self, index: usize) -> &Self::Output {
//         // todo(mb): assert conditions
//         let start = self.offset[index].try_into().unwrap();
//         let end = self.offset[index + 1].try_into().unwrap();
//         &self.data[start..end]
//     }
// }

// /// Iterator over elements of an array with variable-sized binary data.
// // todo(mb): impl nth and advance_by
// pub struct VariableSizeBinaryArrayIter<'a, T, const N: bool>
// where
//     T: OffsetValue,
//     &'a Offset<T, N>: IntoIterator,
// {
//     data: &'a [u8],
//     offset: <&'a Offset<T, N> as IntoIterator>::IntoIter,
// }

// impl<'a, T> Iterator for VariableSizeBinaryArrayIter<'a, T, false>
// where
//     T: OffsetValue,
//     &'a Offset<T, false>: IntoIterator<Item = (usize, usize)>,
// {
//     type Item = &'a [u8];

//     fn next(&mut self) -> Option<Self::Item> {
//         self.offset
//             .next()
//             .map(|(start, end)| &self.data[start..end])
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.offset.size_hint()
//     }
// }

// impl<'a, T> IntoIterator for &'a VariableSizeBinaryArray<T, false>
// where
//     T: OffsetValue,
//     &'a Offset<T, false>: IntoIterator<Item = (usize, usize)>,
// {
//     type Item = &'a [u8];
//     type IntoIter = VariableSizeBinaryArrayIter<'a, T, false>;

//     fn into_iter(self) -> Self::IntoIter {
//         VariableSizeBinaryArrayIter {
//             data: &self.data,
//             offset: self.offset.into_iter(),
//         }
//     }
// }

// impl<'a, T> Iterator for VariableSizeBinaryArrayIter<'a, T, true>
// where
//     T: OffsetValue,
//     &'a Offset<T, true>: IntoIterator<Item = Option<(usize, usize)>>,
// {
//     type Item = Option<&'a [u8]>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.offset
//             .next()
//             .map(|opt| opt.map(|(start, end)| &self.data[start..end]))
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.offset.size_hint()
//     }
// }

// impl<'a, T> IntoIterator for &'a VariableSizeBinaryArray<T, true>
// where
//     T: OffsetValue,
//     &'a Offset<T, true>: IntoIterator<Item = Option<(usize, usize)>>,
// {
//     type Item = Option<&'a [u8]>;
//     type IntoIter = VariableSizeBinaryArrayIter<'a, T, true>;

//     fn into_iter(self) -> Self::IntoIter {
//         VariableSizeBinaryArrayIter {
//             data: &self.data,
//             offset: self.offset.into_iter(),
//         }
//     }
// }

/// Array with variable sized binary data. Uses [i32] offsets.
pub type BinaryArray<const N: bool> = VariableSizeBinaryArray<i32, N>;

/// Array with variable sized binary data. Uses [i64] offsets.
pub type LargeBinaryArray<const N: bool> = VariableSizeBinaryArray<i64, N>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Length, Null};

    #[test]
    fn from_iter() {
        let x = vec![1u8, 2, 3, 4, 5];
        let y = vec![1u8, 2, 3, 4];
        let z = vec![1u8, 2, 3];
        let vec = vec![x, y, z];

        let array = vec.clone().into_iter().collect::<BinaryArray<false>>();
        assert_eq!(array.len(), 3);
        assert!(array.all_valid());
        let array = vec.into_iter().collect::<BinaryArray<false>>();
        assert_eq!(array.len(), 3);
        assert!(array.all_valid());
        // assert_eq!(array.data().len(), 12);
        // assert_eq!(&array[0], &x[..]);
        // assert_eq!(&array[1], &y[..]);
        // assert_eq!(&array[2], &z[..]);

        let x = vec![1u8, 2, 3, 4, 5];
        let y = vec![1u8, 2, 3, 4];
        let vec = vec![Some(x.clone()), Some(y), None, None, Some(x), Some(vec![])];
        let array = vec.into_iter().collect::<LargeBinaryArray<true>>();
        dbg!(&array);
        assert_eq!(array.len(), 6);
    }

    #[test]
    fn into_iter() {
        let x = vec![1u8, 2, 3, 4, 5];
        let y = vec![1u8, 2, 3, 4];
        let z = vec![1u8, 2, 3];
        let vec = vec![x, y, z];

        let array = vec.clone().into_iter().collect::<BinaryArray<false>>();
        let mut iter = array.into_iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next(), Some(x));
        assert_eq!(iter.next(), Some(y));
        assert_eq!(iter.next(), Some(z));
        assert_eq!(iter.next(), None);

        let x = vec![1u8, 2, 3, 4, 5];
        let y = vec![1u8, 2, 3, 4];
        let vec = vec![
            Some(x.clone()),
            Some(y),
            None,
            None,
            Some(x.clone()),
            Some(vec![]),
        ];
        let array = vec.into_iter().collect::<LargeBinaryArray<true>>();
        let mut iter = array.into_iter();
        assert_eq!(iter.size_hint(), (6, Some(6)));
        assert_eq!(iter.next(), Some(Some(x)));
        assert_eq!(iter.next(), Some(Some(y)));
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.next(), Some(Some(x)));
        assert_eq!(iter.next(), Some(Some([] as &[u8])));
        assert_eq!(iter.next(), None);
    }
}
