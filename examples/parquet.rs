use std::time::Duration;

use uuid::Uuid;

fn main() {
    use arrow_array::RecordBatch;
    use arrow_cast::pretty;
    use bytes::Bytes;
    use narrow::{
        array::StructArray,
        arrow::{buffer_builder::ArrowBufferBuilder, scalar_buffer::ArrowScalarBuffer},
        ArrayType,
    };
    use parquet::arrow::{arrow_reader::ParquetRecordBatchReader, ArrowWriter};

    #[derive(ArrayType, Default)]
    struct Bar(Option<bool>);

    #[derive(ArrayType, Default)]
    struct Foo {
        a: u32,
        b: Option<u8>,
        c: bool,
        d: String,
        e: Option<Vec<Option<bool>>>,
        f: Bar,
        g: [u8; 8],
        h: Uuid,
        i: Duration,
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
            h: Uuid::max(),
            i: Duration::from_secs(1234),
        },
        Foo {
            a: 42,
            b: None,
            c: false,
            d: "narrow".to_string(),
            e: None,
            f: Bar(None),
            g: [9, 10, 11, 12, 13, 14, 15, 16],
            h: Uuid::nil(),
            i: Duration::from_nanos(1234),
        },
    ];

    let narrow_array = input
        .into_iter()
        .collect::<StructArray<Foo, false, ArrowBufferBuilder>>();

    let record_batch = RecordBatch::from(narrow_array);
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

    println!(
        "From Arrow RecordBatch (via Parquet) to narrow StructArray and back to Arrow RecordBatch"
    );
    let round_trip: StructArray<Foo, false, ArrowScalarBuffer> = read.into();
    println!(
        "Extract field `h` as Uuids: {:?}",
        round_trip.0.h.into_iter().collect::<Box<[Uuid]>>()
    );
    println!(
        "Extract field `i` as Durations: {:?}",
        round_trip.0.i.into_iter().collect::<Box<[Duration]>>()
    );
    let arrow_struct_array_round_trip = arrow_array::StructArray::from(round_trip);
    let record_batch_round_trip = arrow_array::RecordBatch::from(arrow_struct_array_round_trip);
    pretty::print_batches(&[record_batch_round_trip]).unwrap();
}
