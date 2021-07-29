use crate::{
    Array, ArrayIndex, ArrayType, Offset, OffsetValue, VariableSizeBinaryArray,
    VariableSizeBinaryArrayIter,
};
use std::{
    iter::{FromIterator, Map},
    ops::{Deref, Index},
};

/// Array with variable sized string (UTF-8) data.
#[derive(Debug)]
pub struct StringArray<T, const N: bool>(VariableSizeBinaryArray<T, N>)
where
    T: OffsetValue;

/// Array with UTF-8 strings. Uses [i32] offsets.
pub type Utf8Array<const N: bool> = StringArray<i32, N>;

/// Array with UTF-8 strings. Uses [i64] offsets.
pub type LargeUtf8Array<const N: bool> = StringArray<i64, N>;

impl<T, const N: bool> Array for StringArray<T, N>
where
    T: OffsetValue,
{
    type Validity = Offset<T, N>;

    fn validity(&self) -> &Self::Validity {
        self.0.validity()
    }
}

impl<T> ArrayIndex<usize> for StringArray<T, false>
where
    T: OffsetValue,
{
    type Output = String;

    fn index(&self, index: usize) -> Self::Output {
        unsafe { String::from_utf8_unchecked(ArrayIndex::index(&self.0, index)) }
    }
}

impl ArrayType for String {
    type Array = Utf8Array<false>;
}

impl ArrayType for Option<String> {
    type Array = Utf8Array<true>;
}

impl ArrayType for &str {
    type Array = Utf8Array<false>;
}

impl ArrayType for Option<&str> {
    type Array = Utf8Array<true>;
}

impl<T, const N: bool> Deref for StringArray<T, N>
where
    T: OffsetValue,
{
    type Target = VariableSizeBinaryArray<T, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, U> FromIterator<U> for StringArray<T, false>
where
    T: OffsetValue,
    U: AsRef<str> + AsRef<[u8]>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = U>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<T, U> FromIterator<Option<U>> for StringArray<T, true>
where
    T: OffsetValue,
    U: AsRef<str> + AsRef<[u8]>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<U>>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<T> Index<usize> for StringArray<T, false>
where
    T: OffsetValue,
{
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        // todo(mb): bounds
        // Safety
        // - String data is always valid utf-8
        unsafe { std::str::from_utf8_unchecked(Index::index(&self.0, index)) }
    }
}

type StringArrayIter<'a, T, U, V, const N: bool> =
    Map<VariableSizeBinaryArrayIter<'a, T, N>, fn(U) -> V>;

impl<'a, T> IntoIterator for &'a StringArray<T, false>
where
    T: OffsetValue,
{
    type Item = &'a str;
    type IntoIter = StringArrayIter<'a, T, &'a [u8], &'a str, false>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(|slice|
          // Safety
          // - String array must contain valid utf8.           
          unsafe { std::str::from_utf8_unchecked(slice) })
    }
}

impl<'a, T> IntoIterator for &'a StringArray<T, true>
where
    T: OffsetValue,
{
    type Item = Option<&'a str>;
    type IntoIter = StringArrayIter<'a, T, Option<&'a [u8]>, Option<&'a str>, true>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(|opt| {
            opt.map(|slice|
          // Safety
          // - String array must contain valid utf8.           
          unsafe { std::str::from_utf8_unchecked(slice) })
        })
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
        let array = vec.iter().collect::<Utf8Array<false>>();
        assert_eq!(array.len(), 3);
        assert_eq!(array.data().len(), 11);
        assert_eq!(&array[0], x);
        assert_eq!(&array[1], y);
        assert_eq!(&array[2], z);
        let out = array.into_iter().collect::<Vec<_>>();
        assert_eq!(vec, out);
        let hello_world = array.into_iter().collect::<String>();
        assert_eq!(hello_world, "helloworld!");

        let x = "hello";
        let y = "world!";
        let vec = vec![Some(x), Some(y), None, None, Some(x), Some("")];
        let array = vec.iter().copied().collect::<LargeUtf8Array<true>>();
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
        let array = vec.iter().collect::<Utf8Array<false>>();
        let mut iter = array.into_iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next(), Some(x));
        assert_eq!(iter.next(), Some(y));
        assert_eq!(iter.next(), Some(z));
        assert_eq!(iter.next(), None);

        let x = "hello";
        let y = "world";
        let vec = vec![Some(x), Some(y), None, None, Some(x), Some("")];
        let array = vec.into_iter().collect::<Utf8Array<true>>();
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
