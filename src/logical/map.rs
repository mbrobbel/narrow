#![allow(missing_docs)]

use std::{collections::HashMap, hash::Hash};

use crate::{
    array::{self, UnionType},
    buffer::BufferType,
    offset::OffsetElement,
    ArrayType,
};

use super::{LogicalArray, LogicalArrayType};

impl<K: array::ArrayType<K> + Eq + Hash, V: array::ArrayType<V>> array::ArrayType<HashMap<K, V>>
    for HashMap<K, V>
{
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<Self, false, Buffer, OffsetItem, UnionLayout>;
}

impl<K: array::ArrayType<K> + Eq + Hash, V: array::ArrayType<V>> array::ArrayType<HashMap<K, V>>
    for Option<HashMap<K, V>>
{
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<HashMap<K, V>, true, Buffer, OffsetItem, UnionLayout>;
}

// TODO(mbrobbel): support HashMap<K, Option<V>>

/// An item in a map.
#[derive(ArrayType)]
pub struct KeyValue<K, V> {
    /// The key.
    key: K,
    /// The value.
    value: V,
}

impl<K: array::ArrayType<K> + Hash + Eq, V: array::ArrayType<V>> LogicalArrayType<HashMap<K, V>>
    for HashMap<K, V>
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

/// An array for [`HashMap`] items.
#[allow(unused)]
pub type HashMapArray<
    K,
    V,
    const NULLABLE: bool = false,
    Buffer = crate::buffer::VecBuffer,
    OffsetItem = i32,
> = LogicalArray<HashMap<K, V>, NULLABLE, Buffer, OffsetItem, crate::array::union::NA>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Length;

    #[test]
    fn from_iter() {
        let array = [
            HashMap::default(),
            HashMap::from_iter([("a".to_string(), 1), ("b".to_string(), 2)]),
        ]
        .into_iter()
        .collect::<HashMapArray<String, u8>>();
        assert_eq!(array.len(), 2);
        assert_eq!(array.0.len(), 2);

        let array_nullable = [
            Some(HashMap::from_iter([
                ("a".to_string(), 1),
                ("b".to_string(), 2),
            ])),
            None,
        ]
        .into_iter()
        .collect::<HashMapArray<String, i8, true>>();
        assert_eq!(array_nullable.len(), 2);
        assert_eq!(array_nullable.0.len(), 2);
    }

    #[test]
    fn into_iter() {
        let input = [
            HashMap::default(),
            HashMap::from_iter([("a".to_string(), 1), ("b".to_string(), 2)]),
        ];
        let array = input
            .clone()
            .into_iter()
            .collect::<HashMapArray<String, i32>>();
        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let input_nullable = [
            Some(HashMap::from_iter([
                ("a".to_string(), 1),
                ("b".to_string(), 2),
            ])),
            None,
        ];
        let array_nullable = input_nullable
            .clone()
            .into_iter()
            .collect::<HashMapArray<String, u64, true>>();
        let output_nullable = array_nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(input_nullable, output_nullable.as_slice());
    }
}
