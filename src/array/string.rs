use crate::{
    Array, ArrayType, Buffer, DataBuffer, Length, Null, NullableOffsetIter, OffsetIter,
    OffsetSliceIter, OffsetValue, VariableSizeBinaryArray, DEFAULT_ALIGNMENT,
};
use std::{iter::Map, str};

/// Array with variable sized string (UTF-8) data.
#[derive(Debug)]
pub struct StringArray<T, const N: bool = true, const A: usize = DEFAULT_ALIGNMENT>(
    VariableSizeBinaryArray<T, N, A>,
);

/// Array with UTF-8 strings. Uses [i32] offsets.
pub type Utf8Array<const N: bool = true, const A: usize = DEFAULT_ALIGNMENT> =
    StringArray<i32, N, A>;

/// Array with UTF-8 strings. Uses [i64] offsets.
pub type LargeUtf8Array<const N: bool = true, const A: usize = DEFAULT_ALIGNMENT> =
    StringArray<i64, N, A>;

impl<T, const N: bool, const A: usize> Array for StringArray<T, N, A>
where
    T: OffsetValue,
{
    type Item<'a> = &'a str;
}

impl ArrayType for String {
    type Item<'a> = &'a str;
    type Array<T, const N: bool, const A: usize> = StringArray<T, false, A>;
}

impl ArrayType for &str {
    type Item<'a> = &'a str;
    type Array<T, const N: bool, const A: usize> = StringArray<T, false, A>;
}

impl<'a, T, const A: usize> FromIterator<&'a str> for StringArray<T, false, A>
where
    T: OffsetValue,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        Self(iter.into_iter().map(|x| x.as_bytes()).collect())
    }
}

impl<'a, T, const A: usize> FromIterator<Option<&'a str>> for StringArray<T, true, A>
where
    T: OffsetValue,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<&'a str>>,
    {
        Self(
            iter.into_iter()
                .map(|opt| opt.map(|x| x.as_bytes()))
                .collect(),
        )
    }
}

impl<T, const N: bool, const A: usize> Length for StringArray<T, N, A>
where
    VariableSizeBinaryArray<T, N, A>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T, const N: bool, const A: usize> Null for StringArray<T, N, A>
where
    VariableSizeBinaryArray<T, N, A>: Null,
{
    unsafe fn is_valid_unchecked(&self, index: usize) -> bool {
        self.0.is_valid_unchecked(index)
    }
}

pub type StringIter<'a, T> =
    Map<OffsetSliceIter<'a, u8, OffsetIter<'a, T>, false>, fn(&'a [u8]) -> &'a str>;

impl<'a, T, const A: usize> IntoIterator for &'a StringArray<T, false, A>
where
    T: OffsetValue,
{
    type Item = &'a str;
    type IntoIter = StringIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0
            .into_iter()
            // Safety
            // - String array must contain valid utf8.
            .map(|slice| unsafe { str::from_utf8_unchecked(slice) })
    }
}

pub type NullableStringIter<'a, T> = Map<
    OffsetSliceIter<'a, u8, NullableOffsetIter<'a, T>, true>,
    fn(Option<&'a [u8]>) -> Option<&'a str>,
>;

impl<'a, T, const A: usize> IntoIterator for &'a StringArray<T, true, A>
where
    T: OffsetValue,
{
    type Item = Option<&'a str>;
    type IntoIter = NullableStringIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0
            .into_iter()
            // Safety
            // - String array must contain valid utf8.
            .map(|opt| opt.map(|slice| unsafe { str::from_utf8_unchecked(slice) }))
    }
}

impl<T, const N: bool, const A: usize> DataBuffer<u8, A> for StringArray<T, N, A> {
    fn data_buffer(&self) -> &Buffer<u8, A> {
        self.0.data_buffer()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        let x = "hello";
        let y = "world";
        let z = "!";
        let vec = vec![x, y, z];
        let array = vec.clone().into_iter().collect::<Utf8Array<false>>();
        assert_eq!(array.len(), 3);
        assert_eq!(array.data_buffer().len(), 11);
        // assert_eq!(&array[0], x);
        // assert_eq!(&array[1], y);
        // assert_eq!(&array[2], z);
        let out = array.into_iter().collect::<Vec<_>>();
        assert_eq!(vec, out);
        let hello_world = array.into_iter().collect::<String>();
        assert_eq!(hello_world, "helloworld!");

        let x = "hello";
        let y = "world!";
        let vec = vec![Some(x), Some(y), None, None, Some(x), Some("")];
        let array = vec.clone().into_iter().collect::<LargeUtf8Array>();
        assert_eq!(array.len(), 6);
        let out = array.into_iter().collect::<Vec<_>>();
        assert_eq!(vec, out);
    }

    #[test]
    fn into_iter() {
        let x = "hello";
        let y = "world";
        let z = "!";
        let vec = vec![x, y, z];
        let array = vec.into_iter().collect::<Utf8Array<false>>();
        let mut iter = array.into_iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next(), Some(x));
        assert_eq!(iter.next(), Some(y));
        assert_eq!(iter.next(), Some(z));
        assert_eq!(iter.next(), None);

        let x = "hello";
        let y = "world";
        let vec = vec![Some(x), Some(y), None, None, Some(x), Some("")];
        let array = vec.into_iter().collect::<Utf8Array>();
        let mut iter = array.into_iter();
        assert_eq!(iter.size_hint(), (6, Some(6)));
        assert_eq!(iter.next(), Some(Some(x)));
        assert_eq!(iter.next(), Some(Some(y)));
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.next(), Some(Some(x)));
        assert_eq!(iter.next(), Some(Some("")));
        assert_eq!(iter.next(), None);
    }
}
