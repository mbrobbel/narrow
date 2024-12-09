//! Logical array support.

use std::iter::Map;

use crate::{
    array::{Array, ArrayType, ArrayTypeOf, SparseLayout, UnionType},
    buffer::{BufferType, VecBuffer},
    nullability::NonNullable,
    offset::Offset,
    Length, Nullability, Nullable,
};

/// Box support via logical arrays.
pub mod r#box;

#[cfg(feature = "chrono")]
/// Chrono support via logical arrays.
pub mod chrono;

/// Map arrays via logical arrays.
#[cfg(feature = "map")]
pub mod map;

#[cfg(feature = "uuid")]
/// Uuid support via logical arrays.
pub mod uuid;

/// Types that can be stored in Arrow arrays, but require mapping via
/// [`LogicalArray`].
///
// Note: the generic `T` is required to allow a generic parameter for one type e.g. duration.
pub trait LogicalArrayType<T: ?Sized>: Sized
where
    Self: ArrayType<Self>,
    Option<Self>: ArrayType<Self>,
{
    /// Corresponding [`ArrayType`].
    type ArrayType: ArrayType<Self::ArrayType>;

    /// Convert from [`Self::ArrayType`].
    fn from_array_type(item: Self::ArrayType) -> Self;

    /// Convert into [`Self::ArrayType`].
    fn into_array_type(self) -> Self::ArrayType;
}

/// An array for [`LogicalArrayType`] items, that are stored in Arrow arrays,
/// but convertable from and to theirself via this array wrapper.
#[allow(clippy::type_complexity)]
pub struct LogicalArray<
    T: LogicalArrayType<T>,
    Nullable: Nullability = NonNullable,
    Buffer: BufferType = VecBuffer,
    OffsetItem: Offset = i32,
    UnionLayout: UnionType = SparseLayout,
>(
    pub(crate)  <Nullable::Item<T::ArrayType> as ArrayType<T::ArrayType>>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >, //     <<T as LogicalArrayType<T>>::ArrayType as ArrayType<
       //         <T as LogicalArrayType<T>>::ArrayType,
       //     >>::Array<Buffer, OffsetItem, UnionLayout>,
       // >,
)
where
    Option<T>: ArrayType<T>,
    Nullable::Item<T::ArrayType>: ArrayType<T::ArrayType>;

impl<
        T: LogicalArrayType<T>,
        Nullable: Nullability,
        Buffer: BufferType,
        OffsetItem: Offset,
        UnionLayout: UnionType,
    > Array for LogicalArray<T, Nullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Nullable::Item<T::ArrayType>: ArrayType<T::ArrayType>,
{
    type Item = Nullable::Item<T::ArrayType>;
}

impl<
        T: LogicalArrayType<T>,
        Nullable: Nullability,
        Buffer: BufferType,
        OffsetItem: Offset,
        UnionLayout: UnionType,
    > Clone for LogicalArray<T, Nullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Nullable::Item<T::ArrayType>: ArrayType<T::ArrayType>,
    <Nullable::Item<T::ArrayType> as ArrayType<T::ArrayType>>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<
        T: LogicalArrayType<T>,
        Nullable: Nullability,
        Buffer: BufferType,
        OffsetItem: Offset,
        UnionLayout: UnionType,
    > Default for LogicalArray<T, Nullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Nullable::Item<T::ArrayType>: ArrayType<T::ArrayType>,
    <Nullable::Item<T::ArrayType> as ArrayType<T::ArrayType>>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: LogicalArrayType<T>, Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType>
    Extend<T> for LogicalArray<T, NonNullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    ArrayTypeOf<T::ArrayType, Buffer, OffsetItem, UnionLayout>: Extend<T::ArrayType>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0
            .extend(iter.into_iter().map(LogicalArrayType::into_array_type));
    }
}

impl<T: LogicalArrayType<T>, Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType>
    Extend<Option<T>> for LogicalArray<T, Nullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Option<T::ArrayType>: ArrayType<T::ArrayType>,
    <Option<T::ArrayType> as ArrayType<T::ArrayType>>::Array<Buffer, OffsetItem, UnionLayout>:
        Extend<Option<T::ArrayType>>,
{
    fn extend<I: IntoIterator<Item = Option<T>>>(&mut self, iter: I) {
        self.0.extend(
            iter.into_iter()
                .map(|opt| opt.map(LogicalArrayType::into_array_type)),
        );
    }
}

impl<T: LogicalArrayType<T>, Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType>
    FromIterator<T> for LogicalArray<T, NonNullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    ArrayTypeOf<T::ArrayType, Buffer, OffsetItem, UnionLayout>: FromIterator<T::ArrayType>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(LogicalArrayType::into_array_type)
                .collect(),
        )
    }
}

impl<T: LogicalArrayType<T>, Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType>
    FromIterator<Option<T>> for LogicalArray<T, Nullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Option<T::ArrayType>: ArrayType<T::ArrayType>,
    <Option<T::ArrayType> as ArrayType<T::ArrayType>>::Array<Buffer, OffsetItem, UnionLayout>:
        FromIterator<Option<T::ArrayType>>,
{
    fn from_iter<I: IntoIterator<Item = Option<T>>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|opt| opt.map(LogicalArrayType::into_array_type))
                .collect(),
        )
    }
}

impl<T: LogicalArrayType<T>, Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType>
    IntoIterator for LogicalArray<T, NonNullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    ArrayTypeOf<T::ArrayType, Buffer, OffsetItem, UnionLayout>:
        IntoIterator<Item: Into<T::ArrayType>>,
{
    type Item = T;
    type IntoIter = Map<
        <ArrayTypeOf<T::ArrayType, Buffer, OffsetItem, UnionLayout> as IntoIterator>::IntoIter,
        fn(<ArrayTypeOf<T::ArrayType, Buffer, OffsetItem, UnionLayout> as IntoIterator>::Item) -> T,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0
            .into_iter()
            .map(|item| LogicalArrayType::from_array_type(item.into()))
    }
}

impl<T: LogicalArrayType<T>, Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType>
    IntoIterator for LogicalArray<T, Nullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Option<T::ArrayType>: ArrayType<T::ArrayType>,
    <Option<T::ArrayType> as ArrayType<T::ArrayType>>::Array<Buffer, OffsetItem, UnionLayout>:
        IntoIterator<Item: IntoIterator<Item: Into<T::ArrayType>>>,
{
    type Item = Option<T>;
    type IntoIter =
        Map<
            <<Option<T::ArrayType> as ArrayType<T::ArrayType>>::Array<
                Buffer,
                OffsetItem,
                UnionLayout,
            > as IntoIterator>::IntoIter,
            fn(
                <<Option<T::ArrayType> as ArrayType<T::ArrayType>>::Array<
                    Buffer,
                    OffsetItem,
                    UnionLayout,
                > as IntoIterator>::Item,
            ) -> Option<T>,
        >;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(|opt| {
            opt.into_iter()
                .map(Into::into)
                .next()
                .map(LogicalArrayType::from_array_type)
        })
    }
}

impl<
        T: LogicalArrayType<T>,
        Nullable: Nullability,
        Buffer: BufferType,
        OffsetItem: Offset,
        UnionLayout: UnionType,
    > Length for LogicalArray<T, Nullable, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Nullable::Item<T::ArrayType>: ArrayType<T::ArrayType>,
    <Nullable::Item<T::ArrayType> as ArrayType<T::ArrayType>>::Array<
        Buffer,
        OffsetItem,
        UnionLayout,
    >: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq)]
    struct Foo(u8);
    impl ArrayType<Foo> for Foo {
        type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
            LogicalArray<Foo, NonNullable, Buffer, OffsetItem, UnionLayout>;
    }
    impl ArrayType<Foo> for Option<Foo> {
        type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
            LogicalArray<Foo, Nullable, Buffer, OffsetItem, UnionLayout>;
    }

    impl LogicalArrayType<Foo> for Foo {
        type ArrayType = u8;

        fn from_array_type(item: Self::ArrayType) -> Self {
            Self(item)
        }

        fn into_array_type(self) -> Self::ArrayType {
            self.0
        }
    }

    type FooArray<Nullable = NonNullable, Buffer = VecBuffer> = LogicalArray<Foo, Nullable, Buffer>;

    #[test]
    fn from_iter() {
        let input = [Foo(1), Foo(2), Foo(3), Foo(4)];
        let array = input.into_iter().collect::<FooArray>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0 .0, [1, 2, 3, 4]);

        let input_nullable = [Some(Foo(1)), None, Some(Foo(3)), Some(Foo(4))];
        let array_nullable = input_nullable.into_iter().collect::<FooArray<Nullable>>();
        assert_eq!(array_nullable.len(), 4);
        assert_eq!(array_nullable.0 .0.data, [1, u8::default(), 3, 4]);
        assert_eq!(array_nullable.0 .0.validity, [true, false, true, true]);
    }

    #[test]
    fn into_iter() {
        let input = [Foo(1), Foo(2), Foo(3), Foo(4)];
        let array = input.into_iter().collect::<FooArray>();
        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(output, input);

        let input_nullable = [Some(Foo(1)), None, Some(Foo(3)), Some(Foo(4))];
        let array_nullable = input_nullable.into_iter().collect::<FooArray<Nullable>>();
        let output_nullable = array_nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(output_nullable, input_nullable);
    }
}
