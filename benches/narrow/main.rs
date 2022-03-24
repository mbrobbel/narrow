use criterion::{criterion_group, criterion_main, Criterion};

mod bitmap;

criterion_group! {
  name = narrow;
  config = Criterion::default();
  targets =
    bitmap::bench
}
criterion_main!(narrow);
