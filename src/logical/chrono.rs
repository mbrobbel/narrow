use chrono::{DateTime, TimeZone, Utc};

use crate::{
    array::{Array, ArrayType, FixedSizePrimitiveArray, UnionType},
    buffer::BufferType,
    offset::OffsetElement,
};

use super::{LogicalArray, LogicalArrayType};

impl<T: TimeZone> ArrayType for DateTime<T> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<DateTime<T>, false, Buffer, OffsetItem, UnionLayout>;
}

impl<T: TimeZone> ArrayType for Option<DateTime<T>> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<DateTime<T>, true, Buffer, OffsetItem, UnionLayout>;
}

impl<T: TimeZone> LogicalArrayType for DateTime<T> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        FixedSizePrimitiveArray<i64, false, Buffer>;

    fn convert<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        self,
    ) -> <<Self as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Array>::Item {
        self.timestamp_nanos_opt().expect("out of range")
    }
}

impl<T: TimeZone> LogicalArrayType for Option<DateTime<T>> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        FixedSizePrimitiveArray<i64, true, Buffer>;

    fn convert<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        self,
    ) -> <<Self as LogicalArrayType>::Array<Buffer, OffsetItem, UnionLayout> as Array>::Item {
        self.map(|date_time| date_time.timestamp_nanos_opt().expect("out of range"))
    }
}

impl<T: TimeZone, Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>
    From<LogicalArray<DateTime<T>, false, Buffer, OffsetItem, UnionLayout>>
    for FixedSizePrimitiveArray<i64, false, Buffer>
{
    fn from(value: LogicalArray<DateTime<T>, false, Buffer, OffsetItem, UnionLayout>) -> Self {
        value.0
    }
}

/// An array for [`DateTime`] items.
#[allow(unused)]
pub type DateTimeArray<T = Utc, const NULLABLE: bool = false, Buffer = crate::buffer::VecBuffer> =
    LogicalArray<DateTime<T>, NULLABLE, Buffer, crate::offset::NA, crate::array::union::NA>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Length;

    #[test]
    fn from_iter() {
        let array = [DateTime::<Utc>::UNIX_EPOCH, DateTime::<Utc>::UNIX_EPOCH]
            .into_iter()
            .collect::<DateTimeArray>();
        assert_eq!(array.len(), 2);
        assert_eq!(array.0.len(), 2);
    }
}
