//! Support storing Uuid items in Arrow arrays.

use super::{LogicalArray, LogicalArrayType, LogicalFrom};
use crate::{
    array::{union, Array, ArrayType, FixedSizeListArray, FixedSizePrimitiveArray, UnionType},
    buffer::BufferType,
    offset::{self, OffsetElement},
};
use uuid::Uuid;

impl ArrayType for Uuid {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<Uuid, false, Buffer, OffsetItem, UnionLayout>;
}
impl ArrayType for Option<Uuid> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<Uuid, true, Buffer, OffsetItem, UnionLayout>;
}

impl LogicalArrayType for Uuid {
    type ArrayLayout<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        FixedSizeListArray<16, FixedSizePrimitiveArray<u8, false, Buffer>, false, Buffer>;

    fn convert_into<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        self,
    ) -> <Self::ArrayLayout<Buffer, OffsetItem, UnionLayout> as Array>::Item {
        self.into_bytes()
    }

    fn convert_from<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        value: <Self::ArrayLayout<Buffer, OffsetItem, UnionLayout> as Array>::Item,
    ) -> Self {
        Self::from_bytes(value)
    }
}

impl LogicalArrayType for Option<Uuid> {
    type ArrayLayout<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        FixedSizeListArray<16, FixedSizePrimitiveArray<u8, false, Buffer>, true, Buffer>;

    fn convert_into<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        self,
    ) -> <Self::ArrayLayout<Buffer, OffsetItem, UnionLayout> as Array>::Item {
        self.map(Uuid::into_bytes)
    }

    fn convert_from<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        value: <Self::ArrayLayout<Buffer, OffsetItem, UnionLayout> as Array>::Item,
    ) -> Self {
        value.map(Uuid::from_bytes)
    }
}

/// An array for Uuid items.
pub type UuidArray<const NULLABLE: bool = false, Buffer = crate::buffer::VecBuffer> =
    LogicalArray<Uuid, NULLABLE, Buffer, offset::NA, union::NA>;

impl LogicalFrom<[&u8; 16]> for Uuid {
    fn from(value: [&u8; 16]) -> Self {
        Uuid::from_bytes(value.map(ToOwned::to_owned))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        let input = [Uuid::from_u128(u128::MIN), Uuid::from_u128(u128::MAX)];
        let uuid_array = input.into_iter().collect::<UuidArray>();
        assert_eq!(
            uuid_array.0 .0 .0,
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff,
            ]
        );
        let array = &uuid_array;
        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());
    }
}
