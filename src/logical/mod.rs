//! Logical array support.

use crate::{
    array::{Array, ArrayType, UnionType},
    buffer::BufferType,
    offset::OffsetElement,
    validity::{Nullability, Validity},
};

/// Types that can be stored in Arrow arrays, but require mapping via
/// [`LogicalArray`].
pub trait LogicalArrayType: ArrayType {
    /// The Arrow [`Array`] used inside [`LogicalArray`] to store these types.
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>: Array;

    /// Convert an item into the item type of the associated array.
    fn convert<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        self,
    ) -> <<Self as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Array>::Item;
}

/// An array for [`LogicalArrayType`] items, that are stored in Arrow arrays,
/// but convertable from and to theirself via this array wrapper.
pub struct LogicalArray<
    T: LogicalArrayType,
    const NULLABLE: bool,
    Buffer: BufferType,
    OffsetItem: OffsetElement,
    UnionLayout: UnionType,
>(<<T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Validity<NULLABLE>>::Storage<Buffer>
) where <T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>;

impl<
        T: LogicalArrayType,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > Array for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>,
    T: Nullability<NULLABLE>,
{
    type Item = <T as Nullability<NULLABLE>>::Item;
}

impl<
        T: LogicalArrayType,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > FromIterator<T>
    for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>,
    <<T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Validity<NULLABLE>>::Storage<Buffer>: FromIterator<<<T as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Array>::Item>
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(LogicalArrayType::convert).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::{array::union, buffer::VecBuffer, offset};

    use super::*;

    struct Foo(u8);
    impl ArrayType for Foo {
        type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
            LogicalArray<Foo, false, Buffer, OffsetItem, UnionLayout>;
    }
    impl LogicalArrayType for Foo {
        type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
            <u8 as ArrayType>::Array<Buffer, OffsetItem, UnionLayout>;

        fn convert<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
            self,
        ) -> <<Self as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Array>::Item
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
