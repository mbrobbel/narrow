use arrow_array::temporal_conversions::NANOSECONDS;
use chrono::{DateTime, NaiveDateTime, NaiveDate, NaiveTime, TimeDelta, Timelike, Utc, Datelike};

use crate::{
    array::{ArrayType, UnionType},
    buffer::BufferType,
    offset::OffsetElement,
};

use super::{LogicalArray, LogicalArrayType};

impl ArrayType<DateTime<Utc>> for DateTime<Utc> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<Self, false, Buffer, OffsetItem, UnionLayout>;
}

impl ArrayType<DateTime<Utc>> for Option<DateTime<Utc>> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<DateTime<Utc>, true, Buffer, OffsetItem, UnionLayout>;
}

impl LogicalArrayType<DateTime<Utc>> for DateTime<Utc> {
    type ArrayType = i64;

    fn from_array_type(item: Self::ArrayType) -> Self {
        DateTime::from_timestamp_nanos(item)
    }

    fn into_array_type(self) -> Self::ArrayType {
        self.timestamp_nanos_opt().expect("out of range")
    }
}

/// An array for [`DateTime`] items.
pub type DateTimeArray<const NULLABLE: bool = false, Buffer = crate::buffer::VecBuffer> =
    LogicalArray<DateTime<Utc>, NULLABLE, Buffer, crate::offset::NA, crate::array::union::NA>;

impl ArrayType<NaiveDateTime> for NaiveDateTime {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<Self, false, Buffer, OffsetItem, UnionLayout>;
}

impl ArrayType<NaiveDateTime> for Option<NaiveDateTime> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<NaiveDateTime, true, Buffer, OffsetItem, UnionLayout>;
}

impl LogicalArrayType<NaiveDateTime> for NaiveDateTime {
    type ArrayType = i64;

    fn from_array_type(item: Self::ArrayType) -> Self {
        DateTime::from_timestamp_nanos(item).naive_utc()
    }

    fn into_array_type(self) -> Self::ArrayType {
        self.and_utc().timestamp_nanos_opt().expect("out of range")
    }
}

/// An array for [`NaiveDateTime`] items.
pub type NaiveDateTimeArray<const NULLABLE: bool = false, Buffer = crate::buffer::VecBuffer> =
    LogicalArray<NaiveDateTime, NULLABLE, Buffer, crate::offset::NA, crate::array::union::NA>;

impl ArrayType<NaiveDate> for NaiveDate {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
    LogicalArray<Self, false, Buffer, OffsetItem, UnionLayout>;
}

impl ArrayType<NaiveDate> for Option<NaiveDate> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
    LogicalArray<NaiveDate, true, Buffer, OffsetItem, UnionLayout>;
}

impl LogicalArrayType<NaiveDate> for NaiveDate {
    type ArrayType = i32;

    fn from_array_type(item: Self::ArrayType) -> Self {
        NaiveDate::from_num_days_from_ce_opt(item).expect("out of range")
    }

    fn into_array_type(self) -> Self::ArrayType {
        self.num_days_from_ce()
    }
}

/// An array for [`NaiveDate`] items.
pub type NaiveDateArray<const NULLABLE: bool = false, Buffer = crate::buffer::VecBuffer> =
LogicalArray<NaiveDate, NULLABLE, Buffer, crate::offset::NA, crate::array::union::NA>;

impl ArrayType<NaiveTime> for NaiveTime {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<Self, false, Buffer, OffsetItem, UnionLayout>;
}

impl ArrayType<NaiveTime> for Option<NaiveTime> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
        LogicalArray<NaiveTime, true, Buffer, OffsetItem, UnionLayout>;
}

/// The number of nano seconds in a second.
const NANO_SECONDS: i64 = 1_000_000_000;

impl LogicalArrayType<NaiveTime> for NaiveTime {
    type ArrayType = i64;

    fn from_array_type(item: Self::ArrayType) -> Self {
        let (secs, nano) = (item.div_euclid(NANO_SECONDS), item.rem_euclid(NANO_SECONDS));
        Self::from_num_seconds_from_midnight_opt(
            u32::try_from(secs).expect("out of range"),
            u32::try_from(nano).expect("out of range"),
        )
        .expect("out of range")
    }

    fn into_array_type(self) -> Self::ArrayType {
        i64::from(self.num_seconds_from_midnight()) * NANO_SECONDS + i64::from(self.nanosecond())
    }
}

/// An array for [`NaiveTime`] items.
pub type NaiveTimeArray<const NULLABLE: bool = false, Buffer = crate::buffer::VecBuffer> =
    LogicalArray<NaiveTime, NULLABLE, Buffer, crate::offset::NA, crate::array::union::NA>;

impl ArrayType<TimeDelta> for TimeDelta {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
    LogicalArray<Self, false, Buffer, OffsetItem, UnionLayout>;
}

impl ArrayType<TimeDelta> for Option<TimeDelta> {
    type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
    LogicalArray<TimeDelta, true, Buffer, OffsetItem, UnionLayout>;
}

/// The number of nano seconds in a milli second.
const NANOS_PER_MILLI: i64 = 1_000_000;
/// The number of milli seconds in a second.
const MILLI_SECONDS: i64 = 1_000;

impl LogicalArrayType<TimeDelta> for TimeDelta {
    type ArrayType = i64;

    fn from_array_type(item: Self::ArrayType) -> Self {
        let (secs, nano) = (item.div_euclid(MILLI_SECONDS),
                            u32::try_from(item.rem_euclid(NANOS_PER_MILLI))
                                .expect("i64 to u32 cast error"));
        
        Self::new(secs, nano).expect("out of range")
    }

    fn into_array_type(self) -> Self::ArrayType {
        self.num_seconds() * NANOSECONDS + i64::from(self.subsec_nanos())
    }
}

/// An array for [`TimeDelta`] items.
pub type TimeDeltaArray<const NULLABLE: bool = false, Buffer = crate::buffer::VecBuffer> =
LogicalArray<TimeDelta, NULLABLE, Buffer, crate::offset::NA, crate::array::union::NA>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Length;

    #[test]
    fn round_trip_naivedate() {
        for value in [
            NaiveDate::from_yo_opt(2024,7).expect("out of range").num_days_from_ce(),
            NaiveDate::from_yo_opt(2020,6).expect("out of range").num_days_from_ce(),
        ] {
            assert_eq!(NaiveDate::from_num_days_from_ce_opt(value).expect("out of range").into_array_type(), value);
        }
    }

    #[test]
    fn round_trip_naivetime() {
        for value in [
            0,
            1234,
            1234 * NANO_SECONDS,
            86_398 * NANO_SECONDS + 1_999_999_999,
        ] {
            assert_eq!(NaiveTime::from_array_type(value).into_array_type(), value);
        }
    }

    #[test]
    fn round_trip_timedelta() {
        for value in [
            0,
            1234,
            1234 * NANO_SECONDS,
            86_398 * NANO_SECONDS + 1_999_999_999,
        ] {
            assert_eq!(TimeDelta::nanoseconds(value).into_array_type(), value);
        }
    }
    
    #[test]
    fn from_iter() {
        let array = [DateTime::<Utc>::UNIX_EPOCH, DateTime::<Utc>::UNIX_EPOCH]
            .into_iter()
            .collect::<DateTimeArray>();
        assert_eq!(array.len(), 2);
        assert_eq!(array.0.len(), 2);

        let array_nullable = [
            Some(DateTime::<Utc>::UNIX_EPOCH),
            None,
            Some(DateTime::<Utc>::UNIX_EPOCH),
        ]
        .into_iter()
        .collect::<DateTimeArray<true>>();
        assert_eq!(array_nullable.len(), 3);
        assert_eq!(array_nullable.0.len(), 3);
    }

    #[test]
    fn into_iter() {
        let input = [DateTime::<Utc>::UNIX_EPOCH, DateTime::<Utc>::UNIX_EPOCH];
        let array = input.into_iter().collect::<DateTimeArray>();
        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let input_nullable = [
            Some(DateTime::<Utc>::UNIX_EPOCH),
            None,
            Some(DateTime::<Utc>::UNIX_EPOCH),
        ];
        let array_nullable = input_nullable.into_iter().collect::<DateTimeArray<true>>();
        let output_nullable = array_nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(input_nullable, output_nullable.as_slice());
    }
}
