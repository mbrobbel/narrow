fn main() {
    use arrow_array::RecordBatch;
    use arrow_cast::pretty;
    use bytes::Bytes;
    use narrow::{array::StructArray, arrow::buffer_builder::ArrowBufferBuilder, ArrayType};
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
    }
    let input = [
        Foo {
            a: 1,
            b: Some(2),
            c: true,
            d: "hello world!".to_string(),
            e: Some(vec![Some(true), None]),
            f: Bar(Some(true)),
        },
        Foo {
            a: 42,
            b: None,
            c: false,
            d: "narrow".to_string(),
            e: None,
            f: Bar(None),
        },
    ];

    let narrow_array = input
        .into_iter()
        .collect::<StructArray<Foo, false, ArrowBufferBuilder>>();

    let arrow_struct_array = arrow_array::StructArray::from(narrow_array);
    let record_batch = RecordBatch::from(arrow_struct_array);
    pretty::print_batches(&[record_batch.clone()]).unwrap();

    let mut buffer = Vec::new();
    let mut writer = ArrowWriter::try_new(&mut buffer, record_batch.schema(), None).unwrap();
    writer.write(&record_batch).unwrap();
    writer.close().unwrap();

    let mut reader = ParquetRecordBatchReader::try_new(Bytes::from(buffer), 1024).unwrap();
    let read = reader.next().unwrap().unwrap();
    pretty::print_batches(&[read.clone()]).unwrap();
    assert_eq!(record_batch, read);
}
