use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Timelike, Utc};

use crate::{
    NonNullable, Nullable,
    array::{ArrayType, UnionType},
    buffer::BufferType,
    offset::Offset,
};

use super::{LogicalArray, LogicalArrayType};

impl ArrayType<DateTime<Utc>> for DateTime<Utc> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<Self, NonNullable, Buffer, OffsetItem, UnionLayout>;
}

impl ArrayType<DateTime<Utc>> for Option<DateTime<Utc>> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<DateTime<Utc>, Nullable, Buffer, OffsetItem, UnionLayout>;
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

#[cfg(feature = "arrow-rs")]
impl crate::arrow::LogicalArrayType<DateTime<Utc>> for DateTime<Utc> {
    type ExtensionType = crate::arrow::NoExtensionType;
}

/// An array for [`DateTime`] items.
pub type DateTimeArray<Nullable = NonNullable, Buffer = crate::buffer::VecBuffer> =
    LogicalArray<DateTime<Utc>, Nullable, Buffer>;

impl ArrayType<NaiveDateTime> for NaiveDateTime {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<Self, NonNullable, Buffer, OffsetItem, UnionLayout>;
}

impl ArrayType<NaiveDateTime> for Option<NaiveDateTime> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<NaiveDateTime, Nullable, Buffer, OffsetItem, UnionLayout>;
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

#[cfg(feature = "arrow-rs")]
impl crate::arrow::LogicalArrayType<NaiveDateTime> for NaiveDateTime {
    type ExtensionType = crate::arrow::NoExtensionType;
}

/// An array for [`NaiveDateTime`] items.
pub type NaiveDateTimeArray<Nullable = NonNullable, Buffer = crate::buffer::VecBuffer> =
    LogicalArray<NaiveDateTime, Nullable, Buffer>;

impl ArrayType<NaiveDate> for NaiveDate {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<Self, NonNullable, Buffer, OffsetItem, UnionLayout>;
}

impl ArrayType<NaiveDate> for Option<NaiveDate> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<NaiveDate, Nullable, Buffer, OffsetItem, UnionLayout>;
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

#[cfg(feature = "arrow-rs")]
impl crate::arrow::LogicalArrayType<NaiveDate> for NaiveDate {
    type ExtensionType = crate::arrow::NoExtensionType;
}

/// An array for [`NaiveDate`] items.
pub type NaiveDateArray<Nullable = NonNullable, Buffer = crate::buffer::VecBuffer> =
    LogicalArray<NaiveDate, Nullable, Buffer>;

impl ArrayType<NaiveTime> for NaiveTime {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<Self, NonNullable, Buffer, OffsetItem, UnionLayout>;
}

impl ArrayType<NaiveTime> for Option<NaiveTime> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<NaiveTime, Nullable, Buffer, OffsetItem, UnionLayout>;
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

#[cfg(feature = "arrow-rs")]
impl crate::arrow::LogicalArrayType<NaiveTime> for NaiveTime {
    type ExtensionType = crate::arrow::NoExtensionType;
}

/// An array for [`NaiveTime`] items.
pub type NaiveTimeArray<Nullable = NonNullable, Buffer = crate::buffer::VecBuffer> =
    LogicalArray<NaiveTime, Nullable, Buffer>;

impl ArrayType<TimeDelta> for TimeDelta {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<Self, NonNullable, Buffer, OffsetItem, UnionLayout>;
}

impl ArrayType<TimeDelta> for Option<TimeDelta> {
    type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
        LogicalArray<TimeDelta, Nullable, Buffer, OffsetItem, UnionLayout>;
}

impl LogicalArrayType<TimeDelta> for TimeDelta {
    type ArrayType = i64;

    fn from_array_type(item: Self::ArrayType) -> Self {
        Self::nanoseconds(item)
    }

    fn into_array_type(self) -> Self::ArrayType {
        self.num_nanoseconds().expect("out of range")
    }
}

#[cfg(feature = "arrow-rs")]
impl crate::arrow::LogicalArrayType<TimeDelta> for TimeDelta {
    type ExtensionType = crate::arrow::NoExtensionType;
}

/// An array for [`TimeDelta`] items.
pub type TimeDeltaArray<Nullable = NonNullable, Buffer = crate::buffer::VecBuffer> =
    LogicalArray<TimeDelta, Nullable, Buffer>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Length, Nullable};

    #[test]
    fn round_trip_naivedate() {
        for value in [
            NaiveDate::from_yo_opt(2024, 7)
                .expect("out of range")
                .num_days_from_ce(),
            NaiveDate::from_yo_opt(2020, 6)
                .expect("out of range")
                .num_days_from_ce(),
        ] {
            assert_eq!(NaiveDate::from_array_type(value).into_array_type(), value);
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
        .collect::<DateTimeArray<Nullable>>();
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
        let array_nullable = input_nullable
            .into_iter()
            .collect::<DateTimeArray<Nullable>>();
        let output_nullable = array_nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(input_nullable, output_nullable.as_slice());
    }
}
