use criterion::{criterion_group, criterion_main, Criterion};

// mod array;
mod bitmap;
// mod nullable;
// mod compute;

criterion_group! {
  name = narrow;
  config = Criterion::default();
  targets =
    // array::bench,
    bitmap::bench,
    // nullable::bench,
    // compute::bench
}
criterion_main!(narrow);
