//! Array support for durations.

use std::time::Duration;

use crate::{
    array::{union, Array, ArrayType, FixedSizePrimitiveArray, UnionType},
    buffer::{BufferType, VecBuffer},
    offset::{self, OffsetElement},
};

use super::{LogicalArray, LogicalArrayType, LogicalFrom};

// nanoseconds for std::time::Duration?

impl ArrayType for Duration {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<Duration, false, Buffer, OffsetItem, UnionLayout>;
}
impl ArrayType for Option<Duration> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<Duration, true, Buffer, OffsetItem, UnionLayout>;
}
impl LogicalArrayType for Duration {
    type ArrayLayout<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        FixedSizePrimitiveArray<i64, false, Buffer>;

    fn convert_into<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        self,
    ) -> <Self::ArrayLayout<Buffer, OffsetItem, UnionLayout> as Array>::Item {
        i64::try_from(self.as_nanos()).expect("duration (ns) overflow")
    }

    fn convert_from<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType>(
        value: <Self::ArrayLayout<Buffer, OffsetItem, UnionLayout> as Array>::Item,
    ) -> Self {
        Duration::from_nanos(u64::try_from(value).expect("duration (ns) overflow"))
    }
}
///a
pub type DurationArray<const NULLABLE: bool = false, Buffer = VecBuffer> =
    LogicalArray<Duration, NULLABLE, Buffer, offset::NA, union::NA>;

impl LogicalFrom<&i64> for Duration {
    fn from(value: &i64) -> Self {
        Duration::from_nanos(u64::try_from(*value).expect("conversion"))
    }
}
impl LogicalFrom<&&i64> for Duration {
    fn from(value: &&i64) -> Self {
        Duration::from_nanos(u64::try_from(**value).expect("conversion"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        let input = [Duration::from_secs(1), Duration::from_secs(2)];
        let duration_array = input.into_iter().collect::<DurationArray>();
        let array = &duration_array;
        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());
    }
}
