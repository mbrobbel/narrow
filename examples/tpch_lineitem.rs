use arrow_array::{
    builder::{
        FixedSizeListBuilder, Float64Builder, Int64Builder, StringBuilder,
        TimestampNanosecondBuilder, UInt8Builder,
    },
    Array, RecordBatch,
};
use arrow_schema::{DataType, Field, TimeUnit};
use chrono::{DateTime, Utc};
use narrow::{array::StructArray, ArrayType};
use rand::{prelude::SmallRng, Rng, SeedableRng};
use std::sync::Arc;

#[derive(ArrayType, Clone, Debug)]
struct LineItem {
    l_orderkey: i64,
    l_partkey: i64,
    l_suppkey: i64,
    l_linenumber: i64,
    l_quantity: f64,
    l_extendedprice: f64,
    l_discount: f64,
    l_tax: f64,
    l_returnflag: u8,
    l_linestatus: u8,
    l_shipdate: DateTime<Utc>,
    l_commitdate: DateTime<Utc>,
    l_receiptdate: DateTime<Utc>,
    l_shipinstruct: [u8; 25],
    l_shipmode: [u8; 10],
    l_comment: String,
}

// Convert from an iterator of rows to an Arrow RecordBatch:
fn make_recordbatch_narrow(rows: impl Iterator<Item = LineItem>) -> RecordBatch {
    rows.into_iter().collect::<StructArray<LineItem>>().into()
}

// Convert from an iterator of rows to an Arrow RecordBatch:
struct LineItemBuilder {
    l_orderkey: Int64Builder,
    l_partkey: Int64Builder,
    l_suppkey: Int64Builder,
    l_linenumber: Int64Builder,
    l_quantity: Float64Builder,
    l_extendedprice: Float64Builder,
    l_discount: Float64Builder,
    l_tax: Float64Builder,
    l_returnflag: UInt8Builder,
    l_linestatus: UInt8Builder,
    l_shipdate: TimestampNanosecondBuilder,
    l_commitdate: TimestampNanosecondBuilder,
    l_receiptdate: TimestampNanosecondBuilder,
    l_shipinstruct: FixedSizeListBuilder<UInt8Builder>,
    l_shipmode: FixedSizeListBuilder<UInt8Builder>,
    l_comment: StringBuilder,
}

impl Default for LineItemBuilder {
    fn default() -> Self {
        Self {
            l_orderkey: Default::default(),
            l_partkey: Default::default(),
            l_suppkey: Default::default(),
            l_linenumber: Default::default(),
            l_quantity: Default::default(),
            l_extendedprice: Default::default(),
            l_discount: Default::default(),
            l_tax: Default::default(),
            l_returnflag: Default::default(),
            l_linestatus: Default::default(),
            l_shipdate: Default::default(),
            l_commitdate: Default::default(),
            l_receiptdate: Default::default(),
            l_shipinstruct: FixedSizeListBuilder::new(Default::default(), 25),
            l_shipmode: FixedSizeListBuilder::new(Default::default(), 10),
            l_comment: Default::default(),
        }
    }
}

impl LineItemBuilder {
    fn append(&mut self, row: LineItem) {
        self.l_orderkey.append_value(row.l_orderkey);
        self.l_partkey.append_value(row.l_partkey);
        self.l_suppkey.append_value(row.l_suppkey);
        self.l_linenumber.append_value(row.l_linenumber);
        self.l_quantity.append_value(row.l_quantity);
        self.l_extendedprice.append_value(row.l_extendedprice);
        self.l_discount.append_value(row.l_discount);
        self.l_tax.append_value(row.l_tax);
        self.l_returnflag.append_value(row.l_returnflag);
        self.l_linestatus.append_value(row.l_linestatus);
        self.l_shipdate
            .append_option(row.l_shipdate.timestamp_nanos_opt());
        self.l_commitdate
            .append_option(row.l_commitdate.timestamp_nanos_opt());
        self.l_receiptdate
            .append_option(row.l_receiptdate.timestamp_nanos_opt());
        self.l_shipinstruct
            .values()
            .append_values(&row.l_shipinstruct, &[true; 25]);
        self.l_shipinstruct.append(true);
        self.l_shipmode
            .values()
            .append_values(&row.l_shipmode, &[true; 10]);
        self.l_shipmode.append(true);
        self.l_comment.append_value(row.l_comment);
    }

    fn finish(mut self) -> RecordBatch {
        let utc: Arc<str> = Arc::from("UTC");
        let schema = arrow_schema::Schema::new(vec![
            // There is no API to build non-nullable arrays, or convert nullable arrays
            // to non-nullable arrays, so we just use nullable here.
            Field::new("l_orderkey", DataType::Int64, true),
            Field::new("l_partkey", DataType::Int64, true),
            Field::new("l_suppkey", DataType::Int64, true),
            Field::new("l_linenumber", DataType::Int64, true),
            Field::new("l_quantity", DataType::Float64, true),
            Field::new("l_extendedprice", DataType::Float64, true),
            Field::new("l_discount", DataType::Float64, true),
            Field::new("l_tax", DataType::Float64, true),
            Field::new("l_returnflag", DataType::UInt8, true),
            Field::new("l_linestatus", DataType::UInt8, true),
            Field::new(
                "l_shipdate",
                DataType::Timestamp(TimeUnit::Nanosecond, Some(utc.clone())),
                true,
            ),
            Field::new(
                "l_commitdate",
                DataType::Timestamp(TimeUnit::Nanosecond, Some(utc.clone())),
                true,
            ),
            Field::new(
                "l_receiptdate",
                DataType::Timestamp(TimeUnit::Nanosecond, Some(utc.clone())),
                true,
            ),
            Field::new(
                "l_shipinstruct",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::UInt8, true)), 25),
                true,
            ),
            Field::new(
                "l_shipmode",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::UInt8, true)), 10),
                true,
            ),
            Field::new("l_comment", DataType::Utf8, true),
        ]);

        let columns: Vec<Arc<dyn Array>> = vec![
            Arc::new(self.l_orderkey.finish()),
            Arc::new(self.l_partkey.finish()),
            Arc::new(self.l_suppkey.finish()),
            Arc::new(self.l_linenumber.finish()),
            Arc::new(self.l_quantity.finish()),
            Arc::new(self.l_extendedprice.finish()),
            Arc::new(self.l_discount.finish()),
            Arc::new(self.l_tax.finish()),
            Arc::new(self.l_returnflag.finish()),
            Arc::new(self.l_linestatus.finish()),
            Arc::new(self.l_shipdate.with_timezone(utc.clone()).finish()),
            Arc::new(self.l_commitdate.with_timezone(utc.clone()).finish()),
            Arc::new(self.l_receiptdate.with_timezone(utc.clone()).finish()),
            Arc::new(self.l_shipinstruct.finish()),
            Arc::new(self.l_shipmode.finish()),
            Arc::new(self.l_comment.finish()),
        ];

        RecordBatch::try_new(Arc::new(schema), columns).unwrap(/* typically handle errors here too */)
    }
}

fn make_recordbatch_arrow(rows: impl Iterator<Item = LineItem>) -> RecordBatch {
    let mut builder = LineItemBuilder::default();
    rows.for_each(|row| builder.append(row));
    builder.finish()
}

// Create some dummy rows of a given size.
fn make_native_row_oriented(size: usize) -> Vec<LineItem> {
    let mut rng = SmallRng::seed_from_u64(0);

    (0..size)
        .map(|_| LineItem {
            l_orderkey: rng.gen_range(0..i64::MAX),
            l_partkey: rng.gen_range(0..i64::MAX),
            l_suppkey: rng.gen_range(0..i64::MAX),
            l_linenumber: rng.gen_range(0..i64::MAX),
            l_quantity: rng.gen_range(0f64..42f64),
            l_extendedprice: rng.gen_range(0f64..1337f64),
            l_discount: rng.gen_range(0f64..0.1),
            l_tax: rng.gen_range(0f64..0.3),
            l_returnflag: rng.gen_range(0..u8::MAX),
            l_linestatus: rng.gen_range(0..u8::MAX),
            l_shipdate: DateTime::from_timestamp_nanos(rng.gen_range(0..i64::MAX)),
            l_commitdate: DateTime::from_timestamp_nanos(rng.gen_range(0..i64::MAX)),
            l_receiptdate: DateTime::from_timestamp_nanos(rng.gen_range(0..i64::MAX)),
            l_shipinstruct: [rng.gen_range(0..u8::MAX); 25],
            l_shipmode: [rng.gen_range(0..u8::MAX); 10],
            l_comment: String::from_iter(
                (0..rng.gen_range(0..44)).map(|_| rng.gen_range('a'..='z')),
            ),
        })
        .collect()
}

const NUM_ROWS: usize = 1 << 20;

#[rustversion::attr(nightly, allow(non_local_definitions))]
fn main() {
    let input = make_native_row_oriented(NUM_ROWS);

    let narrow = make_recordbatch_narrow(input.clone().into_iter());
    let arrow = make_recordbatch_arrow(input.into_iter());

    // Since nullability differs in the schemas, we can't really compare the entire
    // RecordBatch without doing additional work in removing nullability.
    // assert_eq!(narrow, arrow);

    assert_eq!(narrow.num_rows(), arrow.num_rows());
    assert_eq!(narrow.num_columns(), arrow.num_columns());
}
