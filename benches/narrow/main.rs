use arrow_array::types::{UInt16Type, UInt32Type, UInt64Type, UInt8Type};
use criterion::{criterion_group, criterion_main, Criterion};

mod bitmap;
mod versus;

criterion_group! {
  name = narrow;
  config = Criterion::default();
  targets =
    bitmap::bench,
    versus::arrow::primitive::bench<UInt8Type>,
    versus::arrow::primitive::bench<UInt16Type>,
    versus::arrow::primitive::bench<UInt32Type>,
    versus::arrow::primitive::bench<UInt64Type>,
    versus::arrow::primitive::bench_nullable<UInt8Type>,
    versus::arrow::primitive::bench_nullable<UInt16Type>,
    versus::arrow::primitive::bench_nullable<UInt32Type>,
    versus::arrow::primitive::bench_nullable<UInt64Type>,

}
criterion_main!(narrow);
