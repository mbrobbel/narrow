use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta, Utc};
use narrow::NonNullable;

#[rustversion::attr(nightly, allow(non_local_definitions))]
fn main() {
    use arrow_array::RecordBatch;
    use arrow_cast::pretty;
    use bytes::Bytes;
    use narrow::{
        array::{StructArray, VariableSizeBinary},
        arrow::buffer::ScalarBuffer,
        ArrayType,
    };
    use parquet::arrow::{arrow_reader::ParquetRecordBatchReader, ArrowWriter};
    use uuid::Uuid;

    #[derive(ArrayType, Clone, Debug, Default, PartialEq)]
    struct Bar(Option<bool>);

    #[derive(ArrayType, Clone, Debug, Default, PartialEq)]
    struct Foo {
        a: u32,
        b: Option<u8>,
        c: bool,
        d: String,
        e: Option<Vec<Option<bool>>>,
        f: Bar,
        g: [u8; 8],
        h: Uuid,
        i: VariableSizeBinary,
        j: DateTime<Utc>,
        k: NaiveTime,
        l: Option<HashMap<String, Vec<u8>>>,
        m: NaiveDate,
        n: TimeDelta,
    }
    let input = [
        Foo {
            a: 1,
            b: Some(2),
            c: true,
            d: "hello world!".to_string(),
            e: Some(vec![Some(true), None]),
            f: Bar(Some(true)),
            g: [1, 2, 3, 4, 5, 6, 7, 8],
            h: Uuid::from_u128(1234),
            i: vec![1, 3, 3, 7].into(),
            j: DateTime::UNIX_EPOCH,
            k: NaiveTime::MIN,
            l: Some(HashMap::from_iter([(
                "a".to_string(),
                vec![1, 2, 3, 4, 42],
            )])),
            m: NaiveDate::MAX,
            n: TimeDelta::seconds(12345),
        },
        Foo {
            a: 42,
            b: None,
            c: false,
            d: "narrow".to_string(),
            e: None,
            f: Bar(None),
            g: [9, 10, 11, 12, 13, 14, 15, 16],
            h: Uuid::from_u128(42),
            i: vec![4, 2].into(),
            j: Utc::now(),
            k: Utc::now().time(),
            l: None,
            m: NaiveDate::MIN,
            n: TimeDelta::minutes(1234),
        },
    ];

    let narrow_array = input.clone().into_iter().collect::<StructArray<Foo>>();
    let output = narrow_array.clone().into_iter().collect::<Vec<_>>();
    assert_eq!(input.as_slice(), output);

    let record_batch: RecordBatch = narrow_array.into();
    println!("From narrow StructArray to Arrow RecordBatch");
    pretty::print_batches(&[record_batch.clone()]).unwrap();

    let mut buffer = Vec::new();
    let mut writer = ArrowWriter::try_new(&mut buffer, record_batch.schema(), None).unwrap();
    writer.write(&record_batch).unwrap();
    writer.close().unwrap();

    let mut reader = ParquetRecordBatchReader::try_new(Bytes::from(buffer), 1024).unwrap();
    let read = reader.next().unwrap().unwrap();
    println!("From Arrow RecordBatch to Parquet and back to Arrow RecordBatch");
    pretty::print_batches(&[read.clone()]).unwrap();
    assert_eq!(record_batch, read.clone());

    let round_trip: StructArray<Foo, NonNullable, ScalarBuffer> = read.into();
    let arrow_struct_array_round_trip: arrow_array::StructArray = round_trip.into();
    let record_batch_round_trip = arrow_array::RecordBatch::from(arrow_struct_array_round_trip);
    println!(
        "From Arrow RecordBatch (via Parquet) to narrow StructArray and back to Arrow RecordBatch"
    );
    pretty::print_batches(&[record_batch_round_trip]).unwrap();
}
