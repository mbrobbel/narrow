use crate::{Array, ArrayType, NestedArray, Offset, OffsetValue, Uint8Array};
use std::iter::FromIterator;

/// Array with variable-sized lists of other array types.
///
/// Uses `U` offset types.
/// The const generic parameter `N` indicates nullability of the list items.
#[derive(Debug)]
pub struct VariableSizeListArray<T, U, const N: bool>
where
    T: Array,
    U: OffsetValue,
{
    /// The child array.
    data: T,
    /// The offsets of the lists.
    offset: Offset<U, N>,
}

impl<T> ArrayType for Vec<T>
where
    T: ArrayType,
    // for<'a> &'a <T as ArrayType>::Array: IntoIterator,
{
    // todo(mb): cfg?
    type Array = ListArray<<T as ArrayType>::Array, false>;
}

impl ArrayType for String {
    type Array = ListArray<Uint8Array<false>, false>;
}

impl ArrayType for Option<String> {
    type Array = ListArray<Uint8Array<false>, true>;
}

impl ArrayType for &str {
    type Array = ListArray<Uint8Array<false>, false>;
}

impl ArrayType for Option<&str> {
    type Array = ListArray<Uint8Array<false>, true>;
}

impl<T, U, const N: bool> Array for VariableSizeListArray<T, U, N>
where
    T: Array,
    U: OffsetValue,
{
    type Validity = Offset<U, N>;

    fn validity(&self) -> &Self::Validity {
        &self.offset
    }
}

impl<T, U, const N: bool> NestedArray for VariableSizeListArray<T, U, N>
where
    T: Array,
    U: OffsetValue,
{
    type Child = T;

    fn child(&self) -> &T {
        &self.data
    }
}

/// Array with variable-sized lists of other array types. Uses [i32] offsets.
pub type ListArray<T, const N: bool> = VariableSizeListArray<T, i32, N>;

/// Array with variable-sized lists of other array types. Uses [i64] offsets.
pub type LargeListArray<T, const N: bool> = VariableSizeListArray<T, i64, N>;

impl<T, U, V> FromIterator<V> for VariableSizeListArray<T, U, false>
where
    T: Array + FromIterator<<V as IntoIterator>::Item>,
    U: OffsetValue,
    V: IntoIterator,
    <V as IntoIterator>::IntoIter: Clone,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = V>,
    {
        let iter = iter.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();
        let capacity = upper_bound.unwrap_or(lower_bound);

        let mut offset = Vec::with_capacity(capacity);

        let data = iter
            .into_iter()
            .map(IntoIterator::into_iter)
            .inspect(|iter| offset.push(U::try_from(iter.clone().count()).unwrap()))
            .flatten()
            .collect();

        Self {
            data,
            offset: offset.into_iter().collect(),
        }
    }
}

impl<T, U, V> FromIterator<Option<V>> for VariableSizeListArray<T, U, true>
where
    T: Array + FromIterator<<V as IntoIterator>::Item>,
    U: OffsetValue,
    V: IntoIterator,
    <V as IntoIterator>::IntoIter: Clone,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<V>>,
    {
        let iter = iter.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();
        let capacity = upper_bound.unwrap_or(lower_bound);

        let mut offset = Vec::with_capacity(capacity);

        let data = iter
            .into_iter()
            .filter_map(|opt| match opt {
                Some(iter) => {
                    let iter = iter.into_iter();
                    offset.push(Some(U::try_from(iter.clone().count()).unwrap()));
                    Some(iter)
                }
                None => {
                    offset.push(None);
                    None
                }
            })
            .flatten()
            .collect();

        Self {
            data,
            offset: offset.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BinaryArray, Uint32Array};

    #[test]
    fn from_iter() {
        let vec = vec![vec![1u32, 2, 3, 4], vec![1u32, 2, 3, 4, 5]];
        let list: ListArray<Uint32Array<false>, false> = vec.into_iter().collect();
        assert_eq!(list.len(), 2);
        assert_eq!(&list.child()[..], &[1, 2, 3, 4, 1, 2, 3, 4, 5]);

        let vec = vec![vec![vec![1u32, 2, 3, 4], vec![1u32, 2, 3, 4, 5]]];
        let list: ListArray<ListArray<Uint32Array<false>, false>, false> =
            vec.into_iter().collect();
        assert_eq!(list.len(), 1);
        assert_eq!(list.child().len(), 2);
        assert_eq!(list.child().child().len(), 9);
        assert_eq!(&list.child().child()[..], &[1, 2, 3, 4, 1, 2, 3, 4, 5]);

        let vec = vec![
            Some(vec![1u32, 2, 3, 4]),
            None,
            Some(vec![1u32, 2, 3, 4, 5]),
        ];
        let list: ListArray<Uint32Array<false>, true> = vec.into_iter().collect();
        assert_eq!(list.len(), 3);
        assert_eq!(list.null_count(), 1);
        assert_eq!(list.child().len(), 9);

        let vec = vec![
            vec![Some(1u32), None, Some(3), Some(4)],
            vec![Some(1u32), None, None, Some(4), Some(5)],
        ];
        let list: ListArray<Uint32Array<true>, false> = vec.into_iter().collect();
        assert_eq!(list.len(), 2);
        assert_eq!(list.child().len(), 9);
        assert_eq!(list.child().null_count(), 3);

        let a = vec![Some(1u32), None, Some(2)];
        let b = vec![Some(3), None, Some(4), None];
        let vec = vec![Some(a), None, Some(b)];
        let list: ListArray<Uint32Array<true>, true> = vec.into_iter().collect();
        assert_eq!(list.len(), 3);
        assert_eq!(list.null_count(), 1);
        assert_eq!(list.child().len(), 7);
        assert_eq!(list.child().null_count(), 3);

        let x = vec![1u8, 2, 3, 4, 5];
        let y = vec![1u8, 2, 3, 4];
        let z = vec![1u8, 2, 3];
        let vec = vec![&x[..], &y, &z];
        let input = vec![None, Some(vec.clone()), Some(vec.clone()), None];
        let list: ListArray<BinaryArray<false>, true> = input.into_iter().collect();
        assert_eq!(list.len(), 4);
        assert_eq!(list.null_count(), 2);
        assert_eq!(list.child().len(), 6);
        assert_eq!(list.child().data().len(), 24);
    }
}
