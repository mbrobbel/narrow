//! Logical array support.

use std::iter::Map;

use crate::{
    array::{Array, ArrayType, UnionType},
    buffer::BufferType,
    offset::OffsetElement,
    validity::Nullability,
    Length,
};

#[cfg(feature = "chrono")]
/// Chrono support via logical arrays.
pub mod chrono;

#[cfg(feature = "map")]
/// Map arrays via logical arrays.
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
    const NULLABLE: bool,
    Buffer: BufferType,
    OffsetItem: OffsetElement,
    UnionLayout: UnionType,
>(
    #[rustfmt::skip] // https://github.com/rust-lang/rustfmt/issues/5703
    pub(crate)
        <<<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item as ArrayType<
            <T as LogicalArrayType<T>>::ArrayType,
        >>::Array<Buffer, OffsetItem, UnionLayout>,
)
where
    Option<T>: ArrayType<T>,
    <T as LogicalArrayType<T>>::ArrayType: Nullability<NULLABLE>,
    <<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item:
        ArrayType<<T as LogicalArrayType<T>>::ArrayType>;

impl<
        T: LogicalArrayType<T>,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > Array for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    <T as LogicalArrayType<T>>::ArrayType: Nullability<NULLABLE>,
    <<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item:
        ArrayType<<T as LogicalArrayType<T>>::ArrayType>,
{
    type Item = <<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item;
}

impl<
        T: LogicalArrayType<T>,
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > Default for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    <T as LogicalArrayType<T>>::ArrayType: Nullability<NULLABLE>,
    <<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item:
        ArrayType<<T as LogicalArrayType<T>>::ArrayType>,
    <<<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<
        T: LogicalArrayType<T>,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > Extend<T> for LogicalArray<T, false, Buffer, OffsetItem, UnionLayout>
where
        Option<T>: ArrayType<T>,
    <<T as LogicalArrayType<T>>::ArrayType as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout>: Extend<<T as LogicalArrayType<T>>::ArrayType>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().map(LogicalArrayType::into_array_type));
    }
}

impl<
        T: LogicalArrayType<T>,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > Extend<Option<T>> for LogicalArray<T, true, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Option<<T as LogicalArrayType<T>>::ArrayType>: ArrayType<<T as LogicalArrayType<T>>::ArrayType>,
    <Option<<T as LogicalArrayType<T>>::ArrayType> as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout>:
        Extend<Option<<T as LogicalArrayType<T>>::ArrayType>>,
{
    fn extend<I: IntoIterator<Item = Option<T>>>(&mut self, iter: I) {
        self.0.extend(
            iter.into_iter()
                .map(|opt| opt.map(LogicalArrayType::into_array_type)),
        );
    }
}

impl<
        T: LogicalArrayType<T>,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > FromIterator<T> for LogicalArray<T, false, Buffer, OffsetItem, UnionLayout>
where
        Option<T>: ArrayType<T>,
    <<T as LogicalArrayType<T>>::ArrayType as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout>: FromIterator<<T as LogicalArrayType<T>>::ArrayType>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(LogicalArrayType::into_array_type)
                .collect(),
        )
    }
}

impl<
        T: LogicalArrayType<T>,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > FromIterator<Option<T>> for LogicalArray<T, true, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Option<<T as LogicalArrayType<T>>::ArrayType>: ArrayType<<T as LogicalArrayType<T>>::ArrayType>,
    <Option<<T as LogicalArrayType<T>>::ArrayType> as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout>:
        FromIterator<Option<<T as LogicalArrayType<T>>::ArrayType>>,
{
    fn from_iter<I: IntoIterator<Item = Option<T>>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|opt| opt.map(LogicalArrayType::into_array_type))
                .collect(),
        )
    }
}

impl<
        T: LogicalArrayType<T>,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > IntoIterator for LogicalArray<T, false, Buffer, OffsetItem, UnionLayout>
where
        Option<T>: ArrayType<T>,
    <<T as LogicalArrayType<T>>::ArrayType as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout>: IntoIterator,
    <<<T as LogicalArrayType<T>>::ArrayType as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout> as IntoIterator>::Item: Into<<T as LogicalArrayType<T>>::ArrayType>,
{
    type Item = T;
    type IntoIter = Map<<<<T as LogicalArrayType<T>>::ArrayType as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout> as
        IntoIterator>::IntoIter, fn(<<<T as LogicalArrayType<T>>::ArrayType as ArrayType<
            <T as LogicalArrayType<T>>::ArrayType,
        >>::Array<Buffer, OffsetItem, UnionLayout> as
            IntoIterator>::Item) -> T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(|item| LogicalArrayType::from_array_type(item.into()))
    }
}

impl<
        T: LogicalArrayType<T>,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > IntoIterator for LogicalArray<T, true, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    Option<<T as LogicalArrayType<T>>::ArrayType>: ArrayType<<T as LogicalArrayType<T>>::ArrayType>,
    <Option<<T as LogicalArrayType<T>>::ArrayType> as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout>: IntoIterator,
    <<Option<<T as LogicalArrayType<T>>::ArrayType> as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout> as IntoIterator>::Item: IntoIterator,
    <<<Option<<T as LogicalArrayType<T>>::ArrayType> as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout> as IntoIterator>::Item as IntoIterator>::Item:
        Into<<T as LogicalArrayType<T>>::ArrayType>,
{
    type Item = Option<T>;
    type IntoIter = Map<
        <<Option<<T as LogicalArrayType<T>>::ArrayType> as ArrayType<
            <T as LogicalArrayType<T>>::ArrayType,
        >>::Array<Buffer, OffsetItem, UnionLayout> as IntoIterator>::IntoIter,
        fn(
            <<Option<<T as LogicalArrayType<T>>::ArrayType> as ArrayType<
                <T as LogicalArrayType<T>>::ArrayType,
            >>::Array<Buffer, OffsetItem, UnionLayout> as IntoIterator>::Item,
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
        const NULLABLE: bool,
        Buffer: BufferType,
        OffsetItem: OffsetElement,
        UnionLayout: UnionType,
    > Length for LogicalArray<T, NULLABLE, Buffer, OffsetItem, UnionLayout>
where
    Option<T>: ArrayType<T>,
    <T as LogicalArrayType<T>>::ArrayType: Nullability<NULLABLE>,
    <<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item:
        ArrayType<<T as LogicalArrayType<T>>::ArrayType>,
    <<<T as LogicalArrayType<T>>::ArrayType as Nullability<NULLABLE>>::Item as ArrayType<
        <T as LogicalArrayType<T>>::ArrayType,
    >>::Array<Buffer, OffsetItem, UnionLayout>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::{array::union, buffer::VecBuffer, offset};

    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq)]
    struct Foo(u8);
    impl ArrayType<Foo> for Foo {
        type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
            LogicalArray<Foo, false, Buffer, OffsetItem, UnionLayout>;
    }
    impl ArrayType<Foo> for Option<Foo> {
        type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
            LogicalArray<Foo, true, Buffer, OffsetItem, UnionLayout>;
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

    type FooArray<const NULLABLE: bool = false, Buffer = VecBuffer> =
        LogicalArray<Foo, NULLABLE, Buffer, offset::NA, union::NA>;

    #[test]
    fn from_iter() {
        let input = [Foo(1), Foo(2), Foo(3), Foo(4)];
        let array = input.into_iter().collect::<FooArray>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0 .0, [1, 2, 3, 4]);

        let input_nullable = [Some(Foo(1)), None, Some(Foo(3)), Some(Foo(4))];
        let array_nullable = input_nullable.into_iter().collect::<FooArray<true>>();
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
        let array_nullable = input_nullable.into_iter().collect::<FooArray<true>>();
        let output_nullable = array_nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(output_nullable, input_nullable);
    }
}
