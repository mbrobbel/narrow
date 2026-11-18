//! Dependency-free benchmarks for owned iteration over borrowed and shared
//! slices, the storage of imported arrays.
//!
//! Run with `cargo bench --bench slice`.

mod common;

use std::{hint::black_box, sync::Arc};

use common::bench;
use narrow::collection::Collection;

/// Number of items in each benchmarked slice.
const ITEMS: usize = 1 << 16;

fn main() {
    let values = (0..ITEMS)
        .map(|index| u32::try_from(index).expect("small"))
        .collect::<Vec<_>>();
    let shared: Arc<[u32]> = Arc::from(values.as_slice());
    let options = values
        .iter()
        .map(|&value| (value % 3 != 0).then_some(value))
        .collect::<Vec<_>>();
    let lists = (0..ITEMS >> 4)
        .map(|index| (0..u32::try_from(index & 0xf).expect("small")).collect())
        .collect::<Vec<Vec<u32>>>();

    bench("slice to_vec (baseline)", || {
        black_box(values.as_slice()).to_vec()
    });
    bench("slice into_iter_owned collect", || {
        black_box(values.as_slice())
            .into_iter_owned()
            .collect::<Vec<_>>()
    });
    bench("slice into_iter_owned sum", || {
        black_box(values.as_slice())
            .into_iter_owned()
            .fold(0_u32, u32::wrapping_add)
    });
    bench("arc into_iter_owned collect", || {
        Arc::clone(black_box(&shared))
            .into_iter_owned()
            .collect::<Vec<_>>()
    });
    bench("arc into_iter_owned sum", || {
        Arc::clone(black_box(&shared))
            .into_iter_owned()
            .fold(0_u32, u32::wrapping_add)
    });
    bench("option vec iter_views sum", || {
        black_box(&options)
            .iter_views()
            .flatten()
            .fold(0_u32, u32::wrapping_add)
    });
    bench("option slice into_iter_owned collect", || {
        black_box(options.as_slice())
            .into_iter_owned()
            .collect::<Vec<_>>()
    });
    bench("nested vec iter_views values sum", || {
        black_box(&lists)
            .iter_views()
            .map(|list| list.iter().copied().fold(0_u32, u32::wrapping_add))
            .fold(0_u32, u32::wrapping_add)
    });
    bench("nested vec iter_views len sum", || {
        black_box(&lists)
            .iter_views()
            .map(<[u32]>::len)
            .sum::<usize>()
    });
}
