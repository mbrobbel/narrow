use std::{sync::Arc, time::Instant};

use arrow_array::{
    builder::{FixedSizeListBuilder, Float64Builder, Int64Builder, StringBuilder, UInt8Builder},
    Array, RecordBatch,
};
use arrow_schema::{DataType, Field};
use narrow::{array::StructArray, ArrayType};
use rand::{prelude::SmallRng, Rng, SeedableRng};

#[derive(ArrayType, Debug)]
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
    // TODO: https://github.com/mbrobbel/narrow/issues/165
    // l_shipdate: DateTime,
    // l_commitdate: DateTime,
    // l_receiptdate: DateTime,
    l_shipinstruct: [u8; 25],
    l_shipmode: [u8; 10],
    l_comment: String,
}

// Convert from an iterator of rows to an Arrow RecordBatch in 3 lines of code:
fn make_recordbatch_narrow(rows: impl Iterator<Item = LineItem>) -> RecordBatch {
    rows.into_iter().collect::<StructArray<LineItem>>().into()
}

// Convert from an iterator of rows to an Arrow RecordBatch in 110 lines of code:
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
            l_shipinstruct: [rng.gen_range(0..u8::MAX); 25],
            l_shipmode: [rng.gen_range(0..u8::MAX); 10],
            l_comment: String::from_iter(
                (0..rng.gen_range(0..44)).map(|_| rng.gen_range('a'..='z')),
            ),
        })
        .collect()
}

const NUM_ROWS: usize = 1 << 24;

#[rustversion::attr(nightly, allow(non_local_definitions))]
fn main() {
    let narrow_input = make_native_row_oriented(NUM_ROWS);
    let start = Instant::now();
    let narrow = make_recordbatch_narrow(narrow_input.into_iter());
    let duration = start.elapsed();
    println!("Narrow took: {:?}", duration);

    let arrow_input = make_native_row_oriented(NUM_ROWS);
    let start = Instant::now();
    let arrow = make_recordbatch_arrow(arrow_input.into_iter());
    let duration = start.elapsed();
    println!("Arrow took: {:?}", duration);

    // Since nullability differs in the schemas, we can't really compare the entire
    // RecordBatch.
    // assert_eq!(narrow, arrow);

    assert_eq!(narrow.num_rows(), arrow.num_rows());
    assert_eq!(narrow.num_columns(), arrow.num_columns());
}
