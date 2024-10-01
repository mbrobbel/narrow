use criterion::{criterion_group, criterion_main, Criterion};

mod bitmap;
mod versus;

criterion_group! {
  name = narrow;
  config = Criterion::default();
  targets =
    bitmap::bench,
    versus::arrow::primitive::bench,
}
criterion_main!(narrow);
