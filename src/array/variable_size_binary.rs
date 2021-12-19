use crate::{
    Array, ArrayType, Buffer, DataBuffer, Length, Null, NullableOffsetIter, Offset, OffsetIter,
    OffsetSliceIter, OffsetValue, DEFAULT_ALIGNMENT,
};

/// Array with variable-sized binary data.
#[derive(Debug)]
pub struct VariableSizeBinaryArray<
    T,
    const N: bool = true,
    const A: usize = DEFAULT_ALIGNMENT,
    const B: usize = DEFAULT_ALIGNMENT,
>(Offset<Buffer<u8, A>, T, N, B>);

/// Array with variable sized binary data. Uses [i32] offsets.
pub type BinaryArray<
    const N: bool = true,
    const A: usize = DEFAULT_ALIGNMENT,
    const B: usize = DEFAULT_ALIGNMENT,
> = VariableSizeBinaryArray<i32, N, A, B>;

/// Array with variable sized binary data. Uses [i64] offsets.
pub type LargeBinaryArray<
    const N: bool = true,
    const A: usize = DEFAULT_ALIGNMENT,
    const B: usize = DEFAULT_ALIGNMENT,
> = VariableSizeBinaryArray<i64, N, A, B>;

impl<T, const N: bool, const A: usize, const B: usize> Array for VariableSizeBinaryArray<T, N, A, B>
where
    T: OffsetValue,
{
    type Item<'a> = &'a [u8];
}

impl ArrayType for &[u8] {
    type Item<'a> = &'a [u8];
    type Array<T, const N: bool, const A: usize> = VariableSizeBinaryArray<T, false, A, A>; // todo(mb): AA
}

impl<T, const N: bool, const A: usize, const B: usize> DataBuffer<u8, A>
    for VariableSizeBinaryArray<T, N, A, B>
{
    fn data_buffer(&self) -> &Buffer<u8, A> {
        self.0.data_buffer()
    }
}

impl<T, U, const N: bool, const A: usize, const B: usize> FromIterator<U>
    for VariableSizeBinaryArray<T, N, A, B>
where
    Offset<Buffer<u8, A>, T, N, B>: FromIterator<U>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = U>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<'a, T, const A: usize, const B: usize> IntoIterator
    for &'a VariableSizeBinaryArray<T, false, A, B>
where
    T: OffsetValue,
{
    type Item = &'a [u8];
    type IntoIter = OffsetSliceIter<'a, u8, OffsetIter<'a, T>, false>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_slice()
    }
}

impl<'a, T, const A: usize, const B: usize> IntoIterator
    for &'a VariableSizeBinaryArray<T, true, A, B>
where
    T: OffsetValue,
{
    type Item = Option<&'a [u8]>;
    type IntoIter = OffsetSliceIter<'a, u8, NullableOffsetIter<'a, T>, true>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_slice()
    }
}

impl<T, const N: bool, const A: usize, const B: usize> Length
    for VariableSizeBinaryArray<T, N, A, B>
where
    Offset<Buffer<u8, A>, T, N, B>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T, const N: bool, const A: usize, const B: usize> Null for VariableSizeBinaryArray<T, N, A, B>
where
    Offset<Buffer<u8, A>, T, N, B>: Null,
{
    unsafe fn is_valid_unchecked(&self, index: usize) -> bool {
        self.0.is_valid_unchecked(index)
    }
}

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
        // assert_eq!(array.data_buffer().len(), 12);
        // assert_eq!(&array[0], &x[..]);
        // assert_eq!(&array[1], &y[..]);
        // assert_eq!(&array[2], &z[..]);

        let x = vec![1u8, 2, 3, 4, 5];
        let y = vec![1u8, 2, 3, 4];
        let vec = vec![Some(x.clone()), Some(y), None, None, Some(x), Some(vec![])];
        let array = vec.into_iter().collect::<LargeBinaryArray>();
        dbg!(&array);
        assert_eq!(array.len(), 6);
    }

    #[test]
    fn into_iter() {
        let x = vec![1u8, 2, 3, 4, 5];
        let y = vec![1u8, 2, 3, 4];
        let z = vec![1u8, 2, 3];
        let vec = vec![x.clone(), y.clone(), z.clone()];

        let array = vec.clone().into_iter().collect::<BinaryArray<false>>();
        let mut iter = array.into_iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next(), Some(&x[..]));
        assert_eq!(iter.next(), Some(&y[..]));
        assert_eq!(iter.next(), Some(&z[..]));
        assert_eq!(iter.next(), None);

        let x = vec![1u8, 2, 3, 4, 5];
        let y = vec![1u8, 2, 3, 4];
        let vec = vec![
            Some(x.clone()),
            Some(y.clone()),
            None,
            None,
            Some(x.clone()),
            Some(vec![]),
        ];
        let array = vec.into_iter().collect::<LargeBinaryArray>();
        let mut iter = array.into_iter();
        assert_eq!(iter.size_hint(), (6, Some(6)));
        assert_eq!(iter.next(), Some(Some(&x[..])));
        assert_eq!(iter.next(), Some(Some(&y[..])));
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.next(), Some(Some(&x[..])));
        assert_eq!(iter.next(), Some(Some([].as_slice())));
        assert_eq!(iter.next(), None);
    }
}
