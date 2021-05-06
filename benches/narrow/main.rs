use criterion::{criterion_group, criterion_main, Criterion};

mod array;

criterion_group! {
  name = narrow;
  config = Criterion::default();
  targets =
    array::bench
}
criterion_main!(narrow);
