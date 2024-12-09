use crate::{array::ArrayType, NonNullable, Nullable};

use super::{LogicalArray, LogicalArrayType};

impl<T: ArrayType<T>> ArrayType<Box<T>> for Box<T>
where
    Option<T>: ArrayType<T>,
{
    type Array<
        Buffer: crate::buffer::BufferType,
        OffsetItem: crate::offset::Offset,
        UnionLayout: crate::array::UnionType,
    > = LogicalArray<Self, NonNullable, Buffer, OffsetItem, UnionLayout>;
}

impl<T: ArrayType<T>> ArrayType<Box<T>> for Option<Box<T>>
where
    Option<T>: ArrayType<T>,
{
    type Array<
        Buffer: crate::buffer::BufferType,
        OffsetItem: crate::offset::Offset,
        UnionLayout: crate::array::UnionType,
    > = LogicalArray<Box<T>, Nullable, Buffer, OffsetItem, UnionLayout>;
}

impl<T: ArrayType<T>> LogicalArrayType<Box<T>> for Box<T>
where
    Option<T>: ArrayType<T>,
{
    type ArrayType = T;

    fn from_array_type(item: Self::ArrayType) -> Self {
        Box::new(item)
    }

    fn into_array_type(self) -> Self::ArrayType {
        *self
    }
}

/// An array for [`Box`] items.
#[allow(unused)]
pub type BoxArray<T, Nullable = NonNullable, Buffer = crate::buffer::VecBuffer> =
    LogicalArray<Box<T>, Nullable, Buffer, crate::offset::NA, crate::array::union::NA>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Length;

    #[test]
    fn from_iter() {
        let array = [Box::new(1), Box::new(42)]
            .into_iter()
            .collect::<BoxArray<i32>>();
        assert_eq!(array.len(), 2);
        assert_eq!(array.0.len(), 2);

        let array_nullable = [Some(Box::new(1)), None]
            .into_iter()
            .collect::<BoxArray<i32, Nullable>>();
        assert_eq!(array_nullable.len(), 2);
        assert_eq!(array_nullable.0.len(), 2);
    }

    #[test]
    fn into_iter() {
        let input = [Box::new(1), Box::new(42)];
        let array = input.clone().into_iter().collect::<BoxArray<i32>>();
        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let input_nullable = [Some(Box::new(1)), None];
        let array_nullable = input_nullable
            .clone()
            .into_iter()
            .collect::<BoxArray<i32, Nullable>>();
        let output_nullable = array_nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(input_nullable, output_nullable.as_slice());
    }
}
