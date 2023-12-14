//! Logical array support.

// use std::iter::Map;

use std::iter::Map;

use super::{Array, ArrayType, UnionType};
use crate::{buffer::BufferType, offset::OffsetElement, validity::Validity};

#[cfg(feature = "uuid")]
mod uuid;
#[cfg(feature = "uuid")]
pub use uuid::*;

mod duration;
pub use duration::*;

/// a
pub trait LogicalArrayType: ArrayType {
    /// a
    type ArrayLayout<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>: Array;

    /// Convert into array type items
    fn convert_into<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        self,
    ) -> <Self::ArrayLayout<Buffer, OffsetItem, UnionLayout> as Array>::Item;

    /// Convert from array type items.
    fn convert_from<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        value: <Self::ArrayLayout<Buffer, OffsetItem, UnionLayout> as Array>::Item,
    ) -> Self;
}

/// a
pub trait LogicalFrom<T> {
    /// a
    fn from(value: T) -> Self;
}
// /// a
// pub trait LogicalInto<T> {
//     /// a
//     fn into(value: Self) -> T;
// }

/// workaround for rustfmt issue
type InnerArray<T, const NULLABLE: bool, Buffer, OffsetItem, UnionLayout> = <<T as LogicalArrayType>::ArrayLayout<
    Buffer,
    OffsetItem,
    UnionLayout,
> as Validity<NULLABLE>>::Storage<Buffer>;

/// A wrapper that adds a logical type layer over physical arrays.
pub struct LogicalArray<
    T: LogicalArrayType,
    const NULLABLE: bool,
    Buffer: BufferType,
    OffsetItem: OffsetElement,
    UnionLayout: UnionType,
>(pub(crate) InnerArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>)
where
    <T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>;

impl<
        T: LogicalArrayType,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > Array for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout>: Validity<NULLABLE>,
{
    type Item = T;
}

impl<
        T: LogicalArrayType,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > FromIterator<T> for LogicalArray<T, false, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout>: FromIterator<
        <<T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout> as Array>::Item,
    >,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(LogicalArrayType::convert_into)
                .collect(),
        )
    }
}

impl<
        T: LogicalArrayType,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > Extend<T> for LogicalArray<T, false, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout>: Extend<
        <<T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout> as Array>::Item,
    >,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0
            .extend(iter.into_iter().map(LogicalArrayType::convert_into));
    }
}

impl<
        T: LogicalArrayType,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > Default for LogicalArray<T, false, Buffer, OffsetItem, UnionLayout>
where
    <T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<
        'a,
        T: LogicalArrayType,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > IntoIterator for &'a LogicalArray<T, false, Buffer, OffsetItem, UnionLayout>
where
    &'a <T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout>: IntoIterator,
    T: LogicalFrom<
        <&'a <T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout> as IntoIterator>::Item,
    >,
{
    type Item = T;
    type IntoIter = Map<
        <&'a <T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout> as IntoIterator>::IntoIter,
        fn(
            <&'a <T as LogicalArrayType>::ArrayLayout<Buffer, OffsetItem, UnionLayout> as IntoIterator>::Item,
        ) -> Self::Item,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(LogicalFrom::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        array::{union, Uint8Array},
        buffer::VecBuffer,
        offset,
    };

    use super::*;

    #[derive(Clone, Default, Debug, PartialEq)]
    struct Foo(u8);

    type FooArray<const NULLABLE: bool = false, Buffer = VecBuffer> =
        LogicalArray<Foo, false, Buffer, offset::NA, union::NA>;

    impl ArrayType for Foo {
        type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
            LogicalArray<Foo, false, Buffer, OffsetItem, UnionLayout>;
    }
    impl LogicalArrayType for Foo {
        type ArrayLayout<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
            Uint8Array<false, Buffer>;

        fn convert_into<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
            self,
        ) -> <Self::ArrayLayout<Buffer, OffsetItem, UnionLayout> as Array>::Item {
            self.0
        }

        fn convert_from<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
            value: <Self::ArrayLayout<Buffer, OffsetItem, UnionLayout> as Array>::Item,
        ) -> Self {
            Foo(value)
        }
    }
    impl LogicalFrom<&u8> for Foo {
        fn from(value: &u8) -> Self {
            Foo(*value)
        }
    }

    #[test]
    fn iters() {
        let input = [Foo(1), Foo(2), Foo(3), Foo(4)];
        let array = input.clone().into_iter().collect::<FooArray>();
        let array_ref = &array;
        let back = array_ref.into_iter().collect::<Vec<Foo>>();
        assert_eq!(back, input);
    }
}
