#![allow(clippy::single_element_loop)]

use criterion::{Criterion, criterion_group, criterion_main};

mod bitmap;
mod validity;

criterion_group! {
  name = narrow;
  config = Criterion::default();
  targets =
    bitmap::bench,
    validity::bench
}
criterion_main!(narrow);
