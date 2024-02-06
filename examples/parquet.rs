fn main() {
    use arrow_array::RecordBatch;
    use arrow_cast::pretty;
    use bytes::Bytes;
    use narrow::{
        array::{DenseLayout, SparseLayout, StructArray},
        arrow::{buffer_builder::ArrowBufferBuilder, scalar_buffer::ArrowScalarBuffer},
        ArrayType,
    };
    use parquet::arrow::{arrow_reader::ParquetRecordBatchReader, ArrowWriter};
    use uuid::Uuid;

    #[derive(ArrayType, Default)]
    struct Bar(Option<bool>);

    #[derive(ArrayType, Default)]
    enum FooBar {
        #[default]
        Foo,
        Bar(bool),
        Baz {
            a: u8,
            b: u16,
            c: u32,
        },
    }

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
        i: FooBar,
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
            i: FooBar::Bar(true),
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
            i: FooBar::Baz { a: 1, b: 2, c: 42 },
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

    let round_trip: StructArray<Foo, false, ArrowScalarBuffer> = read.into();
    let arrow_struct_array_round_trip = arrow_array::StructArray::from(round_trip);
    let record_batch_round_trip = arrow_array::RecordBatch::from(arrow_struct_array_round_trip);
    println!(
        "From Arrow RecordBatch (via Parquet) to narrow StructArray and back to Arrow RecordBatch"
    );
    pretty::print_batches(&[record_batch_round_trip]).unwrap();
}
