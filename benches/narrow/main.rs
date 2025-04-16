use criterion::{Criterion, criterion_group, criterion_main};

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
