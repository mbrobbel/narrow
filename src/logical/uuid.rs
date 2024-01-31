use uuid::Uuid;

use crate::{
    array::{Array, ArrayType, FixedSizeListArray, FixedSizePrimitiveArray, UnionType},
    buffer::BufferType,
    offset::OffsetElement,
};

use super::{LogicalArray, LogicalArrayType};

impl ArrayType for Uuid {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<Uuid, false, Buffer, OffsetItem, UnionLayout>;
}

impl ArrayType for Option<Uuid> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<Uuid, true, Buffer, OffsetItem, UnionLayout>;
}

impl LogicalArrayType for Uuid {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        FixedSizeListArray<16, FixedSizePrimitiveArray<u8, false, Buffer>, false, Buffer>;

    fn convert<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        self,
    ) -> <<Self as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Array>::Item {
        self.into_bytes()
    }
}

impl LogicalArrayType for Option<Uuid> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        FixedSizeListArray<16, FixedSizePrimitiveArray<u8, false, Buffer>, true, Buffer>;

    fn convert<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        self,
    ) -> <<Self as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Array>::Item {
        self.map(Uuid::into_bytes)
    }
}

impl<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>
    From<LogicalArray<Uuid, false, Buffer, OffsetItem, UnionLayout>>
    for FixedSizeListArray<16, FixedSizePrimitiveArray<u8, false, Buffer>, false, Buffer>
{
    fn from(value: LogicalArray<Uuid, false, Buffer, OffsetItem, UnionLayout>) -> Self {
        value.0
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
    }
}
