//! Offset abstraction.

use std::{fmt, marker::PhantomData};

use crate::{
    buffer::{BufferType, VecBuffer},
    collection::{Collection, Item},
    fixed_size::FixedSize,
    length::Length,
    nullability::{NonNullable, Nullability},
};

///
pub trait OffsetValue: FixedSize + TryInto<usize> {}

impl OffsetValue for i32 {}
impl OffsetValue for i64 {}

///
#[derive(Debug)]
pub struct Offset<
    T: Collection,
    Nullable: Nullability = NonNullable,
    OffsetItem: OffsetValue = i32,
    Buffer: BufferType = VecBuffer,
> {
    collection: T,
    offsets: Nullable::Collection<Buffer::Buffer<OffsetItem>, Buffer>,
}

impl<T: Collection, Nullable: Nullability, OffsetItem: OffsetValue, Buffer: BufferType> Length
    for Offset<T, Nullable, OffsetItem, Buffer>
{
    fn len(&self) -> usize {
        self.offsets.len()
    }
}

///
#[derive(Debug)]
pub struct Iter<'collection, T: Collection, Nullable: Nullability>(
    PhantomData<&'collection (T, Nullable)>,
);
impl<'collection, T: Collection, Nullable: Nullability> Iterator for Iter<'collection, T, Nullable>
where
    <Nullable as Nullability>::Item<Vec<<T as Collection>::Item>>: Item,
{
    type Item = <Nullable::Item<Vec<<T as Collection>::Item>> as Item>::Ref<'collection>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
///
#[derive(Debug)]
pub struct IntoIter<T: Collection, Nullable: Nullability>(PhantomData<(T, Nullable)>);
impl<T: Collection, Nullable: Nullability> Iterator for IntoIter<T, Nullable> {
    type Item = Nullable::Item<Vec<<T as Collection>::Item>>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

///
#[derive(Debug)]
pub struct Slice<'collection, T: Collection> {
    collection: &'collection T,
    start: usize,
    end: usize,
}

impl<'collection, T: Collection> Iterator for &'collection Slice<'collection, T> {
    type Item = <<T as Collection>::Item as Item>::Ref<'collection>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct OffsetSlice<'offset, T> {
    offset: Offset<T
}

/// An item that is a collection.
pub struct CollectionItem<T: Collection>(T);

impl<T: Collection> Item for CollectionItem<T> {
    type Ref<'collection> = Slice<'collection, T>;

    fn as_ref(&self) -> Self::Ref<'_> {
        let end = self.0.len();
        Slice {
            collection: &self.0,
            start: 0,
            end
        }
    }

    fn to_owned(item: &Self::Ref<'_>) -> Self {
        todo!()
    }
}
impl<T: Item> Item for Vec<T> {
    type Ref<'collection> = &'collection [<T as Item>::Ref<'collection>];

    fn as_ref(&self) -> Self::Ref<'_> {
        self.as_slice()
    }

    fn to_owned(item: &Self::Ref<'_>) -> Self {
        item.into_iter()
            .map(|item| Item::into_owned(item))
            .collect()
    }
}

impl<T: Collection, OffsetItem: OffsetValue, Buffer: BufferType> Collection
    for Offset<T, NonNullable, OffsetItem, Buffer>
{
    type Item = CollectionItem<T>;

    fn index(&self, index: usize) -> Option<<Self::Item as Item>::Ref<'_>> {
        let start = self.offsets.index(index).unwrap();
        let end = self
            .offsets
            .index(index.checked_add(1).expect("overflow"))
            .unwrap();
        Some(Slice {
            collection: &self.collection,
            start,
            end,
        })
    }

    type Iter<'collection>
        = Iter<'collection, T, Nullable>
    where
        Self: 'collection;

    fn iter(&self) -> Self::Iter<'_> {
        Iter(PhantomData)
    }

    type IntoIter = IntoIter<T, Nullable>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(PhantomData)
    }
}

// ///
// #[derive(Debug)]
// pub struct List<T: Item>(Vec<T>);
// impl<T: Item> Item for List<T> {
//     type Ref<'collection> = T::Ref<'collection>;

//     fn as_ref(&self) -> Self::Ref<'_> {
//         self.0.index(0).unwrap()
//     }

//     fn to_owned(item: &Self::Ref<'_>) -> Self {
//         List(vec![])
//     }
// }

// ///
// pub struct OffsetSlice<
//     'collection,
//     T: Collection,
//     Nullable: Nullability,
//     OffsetItem: OffsetValue,
//     Buffer: BufferType,
// > {
//     offset: &'collection Offset<T, Nullable, OffsetItem, Buffer>,
//     start: usize,
//     end: usize,
// }

// impl<T: Item> Item for Vec<T> {
//     type Ref<'collection> = T::Ref<'collection>;

//     fn as_ref(&self) -> Self::Ref<'_> {
//         todo!()
//     }

//     fn to_owned(item: &Self::Ref<'_>) -> Self {
//         todo!()
//     }
// }

// ///
// pub struct OffsetIter<
//     'collection,
//     T: Collection,
//     Nullable: Nullability,
//     OffsetItem: OffsetValue,
//     Buffer: BufferType,
// > {
//     offset: &'collection Offset<T, Nullable, OffsetItem, Buffer>,
//     index: usize,
// }

// impl<'collection, T: Collection, Nullable: Nullability, OffsetItem: OffsetValue, Buffer: BufferType>
//     fmt::Debug for OffsetIter<'collection, T, Nullable, OffsetItem, Buffer>
// {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_tuple("OffsetIter").finish_non_exhaustive()
//     }
// }

// impl<'collection, T: Collection, Nullable: Nullability, OffsetItem: OffsetValue, Buffer: BufferType>
//     Iterator for OffsetIter<'collection, T, Nullable, OffsetItem, Buffer>
// {
//     type Item = <<T as Collection>::Item as Item>::Ref<'collection>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let next = self.offset.index(self.index);
//         self.index += 1;
//         next
//     }
// }

// ///
// pub struct OffsetIntoIter<
//     T: Collection,
//     Nullable: Nullability,
//     OffsetItem: OffsetValue,
//     Buffer: BufferType,
// > {
//     offset: Offset<T, Nullable, OffsetItem, Buffer>,
//     index: usize,
// }

// impl<T: Collection, Nullable: Nullability, OffsetItem: OffsetValue, Buffer: BufferType> fmt::Debug
//     for OffsetIntoIter<T, Nullable, OffsetItem, Buffer>
// {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_tuple("OffsetIntoIter").finish_non_exhaustive()
//     }
// }

// impl<T: Collection, Nullable: Nullability, OffsetItem: OffsetValue, Buffer: BufferType> Iterator
//     for OffsetIntoIter<T, Nullable, OffsetItem, Buffer>
// {
//     type Item = List<<T as Collection>::Item>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let next = self.offset.index(self.index);
//         self.index += 1;
//         next.map(|item| Item::into_owned(item))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection() {
        let offset: Offset<_, NonNullable> = Offset {
            collection: vec![1, 2, 3, 4],
            offsets: vec![0, 1, 2],
        };
        let x = offset.iter().collect::<Vec<_>>();
        dbg!(x);
    }
}
