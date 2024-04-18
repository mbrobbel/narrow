use uuid::Uuid;

use crate::{
    array::{Array, ArrayType, FixedSizeBinaryArray, UnionType},
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
        FixedSizeBinaryArray<16, false, Buffer>;

    fn convert<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        self,
    ) -> <<Self as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Array>::Item {
        self.into_bytes()
    }
}

impl LogicalArrayType for Option<Uuid> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        FixedSizeBinaryArray<16, true, Buffer>;

    fn convert<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        self,
    ) -> <<Self as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Array>::Item {
        self.map(Uuid::into_bytes)
    }
}

impl<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>
    From<LogicalArray<Uuid, false, Buffer, OffsetItem, UnionLayout>>
    for FixedSizeBinaryArray<16, false, Buffer>
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
