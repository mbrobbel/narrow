#![allow(missing_docs)]

use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

use crate::{
    array::{self, SparseLayout, UnionType},
    buffer::BufferType,
    offset::Offset,
    NonNullable, Nullable,
};

use super::{LogicalArray, LogicalArrayType};

impl<K: array::ArrayType<K> + Eq + Hash, V: array::ArrayType<V>, S: BuildHasher + Default>
    array::ArrayType<HashMap<K, V, S>> for HashMap<K, V, S>
{
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<Self, NonNullable, Buffer, OffsetItem, UnionLayout>;
}

impl<K: array::ArrayType<K> + Eq + Hash, V: array::ArrayType<V>, S: BuildHasher + Default>
    array::ArrayType<HashMap<K, V, S>> for Option<HashMap<K, V, S>>
{
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<HashMap<K, V, S>, Nullable, Buffer, OffsetItem, UnionLayout>;
}

// TODO(mbrobbel): support HashMap<K, Option<V>>

/// An item in a map.
#[derive(crate::ArrayType)]
pub struct KeyValue<K, V> {
    /// The key.
    key: K,
    /// The value.
    value: V,
}

impl<K: array::ArrayType<K> + Hash + Eq, V: array::ArrayType<V>, S: BuildHasher + Default>
    LogicalArrayType<HashMap<K, V, S>> for HashMap<K, V, S>
{
    type ArrayType = Vec<KeyValue<K, V>>;

    fn from_array_type(item: Self::ArrayType) -> Self {
        item.into_iter()
            .map(|KeyValue { key, value }| (key, value))
            .collect()
    }

    fn into_array_type(self) -> Self::ArrayType {
        self.into_iter()
            .map(|(key, value)| KeyValue { key, value })
            .collect()
    }
}

#[cfg(feature = "arrow-rs")]
impl<K: array::ArrayType<K> + Hash + Eq, V: array::ArrayType<V>, S: BuildHasher + Default>
    crate::arrow::LogicalArrayType<HashMap<K, V, S>> for HashMap<K, V, S>
{
    type ExtensionType = crate::arrow::NoExtensionType;
}

/// An array for [`HashMap`] items.
#[allow(unused)]
pub type HashMapArray<
    K,
    V,
    Nullable = NonNullable,
    Buffer = crate::buffer::VecBuffer,
    OffsetItem = i32,
    UnionType = SparseLayout,
> = LogicalArray<HashMap<K, V>, Nullable, Buffer, OffsetItem, UnionType>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Length;

    #[test]
    fn from_iter() {
        let array = [
            HashMap::default(),
            HashMap::from_iter([("a".to_owned(), 1), ("b".to_owned(), 2)]),
        ]
        .into_iter()
        .collect::<HashMapArray<String, u8>>();
        assert_eq!(array.len(), 2);
        assert_eq!(array.0.len(), 2);

        let array_nullable = [
            Some(HashMap::from_iter([
                ("a".to_owned(), 1),
                ("b".to_owned(), 2),
            ])),
            None,
        ]
        .into_iter()
        .collect::<HashMapArray<String, i8, Nullable>>();
        assert_eq!(array_nullable.len(), 2);
        assert_eq!(array_nullable.0.len(), 2);
    }

    #[test]
    fn into_iter() {
        let input = [
            HashMap::default(),
            HashMap::from_iter([("a".to_owned(), 1), ("b".to_owned(), 2)]),
        ];
        let array = input
            .clone()
            .into_iter()
            .collect::<HashMapArray<String, i32>>();
        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let input_nullable = [
            Some(HashMap::from_iter([
                ("a".to_owned(), 1),
                ("b".to_owned(), 2),
            ])),
            None,
        ];
        let array_nullable = input_nullable
            .clone()
            .into_iter()
            .collect::<HashMapArray<String, u64, Nullable>>();
        let output_nullable = array_nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(input_nullable, output_nullable.as_slice());
    }
}
