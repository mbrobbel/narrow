use uuid::Uuid;

use crate::{
    array::{ArrayType, FixedSizeBinary, UnionType},
    buffer::BufferType,
    offset::OffsetElement,
};

use super::{LogicalArray, LogicalArrayType};

impl ArrayType<uuid::Uuid> for uuid::Uuid {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<Self, false, Buffer, OffsetItem, UnionLayout>;
}

impl ArrayType<uuid::Uuid> for Option<uuid::Uuid> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<uuid::Uuid, true, Buffer, OffsetItem, UnionLayout>;
}

impl LogicalArrayType<uuid::Uuid> for uuid::Uuid {
    type ArrayType = FixedSizeBinary<16>;

    fn from_array_type(item: Self::ArrayType) -> Self {
        Self::from_bytes(item.into())
    }

    fn into_array_type(self) -> Self::ArrayType {
        Self::into_bytes(self).into()
    }
}

/// An array for [`Uuid`] items.
#[allow(unused)]
pub type UuidArray<const NULLABLE: bool = false, Buffer = crate::buffer::VecBuffer> =
    LogicalArray<Uuid, NULLABLE, Buffer, crate::offset::NA, crate::array::union::NA>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Length;

    #[test]
    fn from_iter() {
        let array = [Uuid::from_u128(1), Uuid::from_u128(42)]
            .into_iter()
            .collect::<UuidArray>();
        assert_eq!(array.len(), 2);
        assert_eq!(array.0.len(), 2);

        let array_nullable = [Some(Uuid::from_u128(1)), None]
            .into_iter()
            .collect::<UuidArray<true>>();
        assert_eq!(array_nullable.len(), 2);
        assert_eq!(array_nullable.0.len(), 2);
    }

    #[test]
    fn into_iter() {
        let input = [Uuid::from_u128(1), Uuid::from_u128(42)];
        let array = input.into_iter().collect::<UuidArray>();
        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let input_nullable = [Some(Uuid::from_u128(1)), None];
        let array_nullable = input_nullable.into_iter().collect::<UuidArray<true>>();
        let output_nullable = array_nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(input_nullable, output_nullable.as_slice());
    }
}
