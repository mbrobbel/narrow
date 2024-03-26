//! Logical array support.

use crate::{
    array::{Array, ArrayType, UnionType},
    buffer::BufferType,
    offset::OffsetElement,
    validity::{Nullability, Validity},
    Length,
};

#[cfg(feature = "uuid")]
/// Uuid support via logical arrays.
mod uuid;

/// Types that can be stored in Arrow arrays, but require mapping via
/// [`LogicalArray`].
///
// Note: the generic `T` is required to allow a generic parameter for one type e.g. duration.
pub trait LogicalArrayType<T: ?Sized>: ArrayType<T> {
    /// The Arrow [`Array`] used inside [`LogicalArray`] to store these types.
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>: Array;

    /// Convert an item into the item type of the associated array.
    fn convert<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        self,
    ) -> <<Self as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout> as Array>::Item;
}

/// An array for [`LogicalArrayType`] items, that are stored in Arrow arrays,
/// but convertable from and to theirself via this array wrapper.
pub struct LogicalArray<
    T: LogicalArrayType<T>,
    const NULLABLE: bool,
    Buffer: BufferType,
    OffsetItem: OffsetElement,
    UnionLayout: UnionType,
>(
    pub(crate)  <<T as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout> as Validity<
        NULLABLE,
    >>::Storage<Buffer>,
)
where
    <T as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>;

impl<
        T: LogicalArrayType<T>,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > Array for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>,
    T: Nullability<NULLABLE>,
{
    type Item = <T as Nullability<NULLABLE>>::Item;
}

impl<
        T: LogicalArrayType<T>,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > Default for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>,
    <<T as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout> as Validity<NULLABLE>>::Storage<Buffer>: Default
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<
        T: LogicalArrayType<T>,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > Extend<T> for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>,
    <<T as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout> as Validity<NULLABLE>>::Storage<Buffer>: Extend<<<T as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout> as Array>::Item>
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().map(LogicalArrayType::convert));
    }
}

impl<
        T: LogicalArrayType<T>,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > FromIterator<T>
    for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>,
    <<T as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout> as Validity<NULLABLE>>::Storage<Buffer>: FromIterator<<<T as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout> as Array>::Item>
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(LogicalArrayType::convert).collect())
    }
}

impl<
        T: LogicalArrayType<T>,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > Length for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
    where
        <T as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>,
        <<T as LogicalArrayType<T>>::Array<Buffer, OffsetItem, UnionLayout> as Validity<NULLABLE>>::Storage<Buffer>: Length
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::{array::union, buffer::VecBuffer, offset};

    use super::*;

    struct Foo(u8);
    impl ArrayType<Self> for Foo {
        type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
            LogicalArray<Foo, false, Buffer, OffsetItem, UnionLayout>;
    }
    impl LogicalArrayType<Foo> for Foo {
        type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
            <u8 as ArrayType<u8>>::Array<Buffer, OffsetItem, UnionLayout>;

        fn convert<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
            self,
        ) -> <<Self as LogicalArrayType<Foo>>::Array<Buffer, OffsetItem, UnionLayout> as Array>::Item
        {
            self.0
        }
    }
    type FooArray<const NULLABLE: bool = false, Buffer = VecBuffer> =
        LogicalArray<Foo, NULLABLE, Buffer, offset::NA, union::NA>;

    #[test]
    fn from_iter() {
        let input = [Foo(1), Foo(2), Foo(3), Foo(4)];
        let array = input.into_iter().collect::<FooArray>();
        assert_eq!(array.0 .0, [1, 2, 3, 4]);
    }
}
