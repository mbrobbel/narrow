use crate::{Array, ArrayIndex, FixedSizePrimitiveArray, NestedArray, Primitive};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    convert::{TryFrom, TryInto},
    fmt::Debug,
    hash::{Hash, Hasher},
    iter::FromIterator,
};

/// Types representing dictionary index values.
///
/// Values with these types can be used to represent index values in an
/// [DictionaryArray].
///
/// This trait is sealed to prevent downstream implementations.
pub trait DictionaryIndexValue:
    Primitive + Hash + Eq + TryInto<usize> + TryFrom<usize> + sealed::Sealed
{
}

mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T where T: super::DictionaryIndexValue {}
}

impl DictionaryIndexValue for i8 {}
impl DictionaryIndexValue for i16 {}
impl DictionaryIndexValue for i32 {}
impl DictionaryIndexValue for i64 {}
impl DictionaryIndexValue for u8 {}
impl DictionaryIndexValue for u16 {}
impl DictionaryIndexValue for u32 {}
impl DictionaryIndexValue for u64 {}
impl DictionaryIndexValue for isize {}
impl DictionaryIndexValue for usize {}

/// Array where values are encoded using a dictionary.
#[derive(Debug)]
pub struct DictionaryArray<T, U, const N: bool>
where
    T: Array,
    U: DictionaryIndexValue,
{
    dictionary: T,
    index: FixedSizePrimitiveArray<U, N>,
}

impl<T, U, const N: bool> Array for DictionaryArray<T, U, N>
where
    T: Array,
    U: DictionaryIndexValue,
{
    type Validity = <FixedSizePrimitiveArray<U, N> as Array>::Validity;

    fn validity(&self) -> &Self::Validity {
        self.index.validity()
    }
}

impl<T, U, const N: bool> NestedArray for DictionaryArray<T, U, N>
where
    T: Array,
    U: DictionaryIndexValue,
{
    type Child = T;

    fn child(&self) -> &Self::Child {
        &self.dictionary
    }
}

impl<T, U, V> FromIterator<V> for DictionaryArray<T, U, false>
where
    T: Array + FromIterator<V>,
    U: DictionaryIndexValue,
    <U as TryFrom<usize>>::Error: Debug,
    V: Eq + Hash,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = V>,
    {
        let mut map = HashMap::new();
        let mut dictionary = Vec::new();

        let index = iter
            .into_iter()
            .map(|item| {
                let mut hasher = DefaultHasher::new();
                item.hash(&mut hasher);
                let key = hasher.finish();

                let len = map.len();
                let index = map.entry(key).or_insert_with(|| {
                    dictionary.push(item);
                    len
                });
                U::try_from(*index).unwrap()
            })
            .collect();

        Self {
            dictionary: dictionary.into_iter().collect(),
            index,
        }
    }
}

// this puts nullability information in the bitmap of the index
impl<T, U, V> FromIterator<Option<V>> for DictionaryArray<T, U, true>
where
    // note that this is FromIterator<V> instead of FromIterator<Option<V>>
    T: Array + FromIterator<V>,
    U: DictionaryIndexValue,
    <U as TryFrom<usize>>::Error: Debug,
    V: Eq + Hash,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<V>>,
    {
        let mut map = HashMap::new();
        let mut dictionary = Vec::new();

        let index = iter
            .into_iter()
            .map(|item| {
                item.map(|item| {
                    let mut hasher = DefaultHasher::new();
                    item.hash(&mut hasher);
                    let key = hasher.finish();

                    let len = map.len();
                    let index = map.entry(key).or_insert_with(|| {
                        dictionary.push(item);
                        len
                    });
                    U::try_from(*index).unwrap()
                })
            })
            .collect();

        Self {
            dictionary: dictionary.into_iter().collect(),
            index,
        }
    }
}

/// Iterator over elements of a dictionary encoded array.
pub struct DictionaryArrayIter<'a, T, U, const N: bool>
where
    U: DictionaryIndexValue,
    &'a FixedSizePrimitiveArray<U, N>: IntoIterator,
{
    index: <&'a FixedSizePrimitiveArray<U, N> as IntoIterator>::IntoIter,
    dictionary: &'a T,
}

impl<'a, T, U> Iterator for DictionaryArrayIter<'a, T, U, false>
where
    T: ArrayIndex<usize>,
    U: DictionaryIndexValue,
    <U as TryInto<usize>>::Error: Debug,
    &'a FixedSizePrimitiveArray<U, false>: IntoIterator<Item = U>,
{
    type Item = <T as ArrayIndex<usize>>::Output;

    fn next(&mut self) -> Option<Self::Item> {
        self.index
            .next()
            .map(|index| self.dictionary.index(index.try_into().unwrap()))
    }
}

impl<'a, T, U> Iterator for DictionaryArrayIter<'a, T, U, true>
where
    T: ArrayIndex<usize>,
    U: DictionaryIndexValue,
    <U as TryInto<usize>>::Error: Debug,
    &'a FixedSizePrimitiveArray<U, true>: IntoIterator<Item = Option<U>>,
{
    type Item = Option<<T as ArrayIndex<usize>>::Output>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index
            .next()
            .map(|opt| opt.map(|index| self.dictionary.index(index.try_into().unwrap())))
    }
}

impl<'a, T, U> IntoIterator for &'a DictionaryArray<T, U, false>
where
    T: Array + ArrayIndex<usize>,
    U: DictionaryIndexValue,
    <U as TryInto<usize>>::Error: Debug,
{
    type Item = <T as ArrayIndex<usize>>::Output;
    type IntoIter = DictionaryArrayIter<'a, T, U, false>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            index: self.index.into_iter(),
            dictionary: &self.dictionary,
        }
    }
}

impl<'a, T, U> IntoIterator for &'a DictionaryArray<T, U, true>
where
    T: Array + ArrayIndex<usize>,
    U: DictionaryIndexValue,
    <U as TryInto<usize>>::Error: Debug,
{
    type Item = Option<<T as ArrayIndex<usize>>::Output>;
    type IntoIter = DictionaryArrayIter<'a, T, U, true>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            index: self.index.into_iter(),
            dictionary: &self.dictionary,
        }
    }
}

// note sure how to generate generic impl for conversion/wrapping, requires GATs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Uint8Array;

    #[test]
    fn from_iter() {
        let vec = vec![1u8, 2, 3, 4, 1, 2, 3, 4, 4, 4, 4];
        let array: DictionaryArray<Uint8Array<false>, u8, false> =
            vec.clone().into_iter().collect();
        assert_eq!(array.len(), vec.len());
        assert_eq!(array.child().len(), 4);

        let vec = vec![
            Some(1u8),
            None,
            Some(2),
            None,
            Some(3),
            None,
            None,
            Some(1),
            Some(2),
        ];
        let array: DictionaryArray<Uint8Array<false>, u32, true> =
            vec.clone().into_iter().collect();
        assert_eq!(array.len(), vec.len());
        assert_eq!(array.child().len(), 3);
        assert!(array.is_null(1));

        let vec = vec![
            Some(Some(1u8)),
            None,
            Some(None),
            None,
            Some(None),
            None,
            None,
            Some(Some(1)),
            Some(None),
        ];
        let array: DictionaryArray<Uint8Array<true>, u32, true> = vec.clone().into_iter().collect();
        assert_eq!(array.len(), vec.len());
        assert_eq!(array.child().len(), 2);
        assert!(array.is_null(1));
    }

    #[test]
    fn into_iter() {
        let vec = vec![1u8, 2, 3, 4, 1, 2, 3, 4, 4, 4, 4];
        let array: DictionaryArray<Uint8Array<false>, u8, false> =
            vec.clone().into_iter().collect();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), vec);

        let vec = vec![
            Some(1u8),
            None,
            Some(2),
            None,
            Some(3),
            None,
            None,
            Some(1),
            Some(2),
        ];
        let array: DictionaryArray<Uint8Array<false>, u32, true> =
            vec.clone().into_iter().collect();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), vec);
    }
}
