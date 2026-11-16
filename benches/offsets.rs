//! Dependency-free benchmarks for variable-length (offset) collections.
//!
//! Run with `cargo bench --bench offsets`.

mod common;

use std::hint::black_box;

use common::bench;
use narrow::{
    collection::{Collection, CollectionAllocIn, CollectionRealloc, owned::IntoOwned},
    length::Length,
    offset::Offsets,
};

/// Number of variable-length items in each benchmarked collection.
const ITEMS: usize = 1 << 16;

/// Number of rows in the byte-row benchmarks.
const BYTE_ROWS: usize = 1 << 8;
/// Length of each row in the byte-row benchmarks.
const BYTE_ROW_LEN: usize = 2 << 10;

/// Deterministic, irregular item length between 0 and 15.
fn item_len(index: usize) -> usize {
    index.wrapping_mul(0x9e37_79b9) >> 11 & 0xf
}

fn main() {
    let lists = (0..ITEMS)
        .map(|index| {
            (0..item_len(index))
                .map(|value| u32::try_from(value).expect("small"))
                .collect()
        })
        .collect::<Vec<Vec<u32>>>();
    let offsets = lists.iter().cloned().collect::<Offsets<Vec<u32>>>();

    bench("offsets view(index) len sum", || {
        let offsets = black_box(&offsets);
        (0..ITEMS)
            .map(|index| offsets.view(index).map_or(0, |item| item.len()))
            .sum::<usize>()
    });
    bench("offsets iter_views len sum", || {
        black_box(&offsets)
            .iter_views()
            .map(|item| item.len())
            .sum::<usize>()
    });
    bench("offsets iter_views borrowed values sum", || {
        black_box(&offsets)
            .iter_views()
            .map(|item| item.iter_views().sum::<u32>())
            .sum::<u32>()
    });
    bench("offsets iter_views owned values sum", || {
        black_box(&offsets)
            .iter_views()
            .flat_map(|item| item.into_iter_owned())
            .sum::<u32>()
    });
    bench("offsets iter_views into_owned", || {
        black_box(&offsets)
            .iter_views()
            .map(IntoOwned::<Vec<u32>>::into_owned)
            .map(|item| item.len())
            .sum::<usize>()
    });
    bench("offsets clone (baseline)", || black_box(&offsets).clone());
    bench("offsets try_from_iter_in 2 KiB byte rows", || {
        let rows = (0..BYTE_ROWS).map(|_| vec![0xab_u8; BYTE_ROW_LEN]);
        <Offsets<Vec<u8>> as CollectionAllocIn>::try_from_iter_in(black_box(rows), ())
    });
    bench("vec try_extend u32", || {
        let mut values = Vec::<u32>::new();
        let items = 0..u32::try_from(ITEMS).expect("fits");
        CollectionRealloc::try_extend(&mut values, black_box(items)).map(|()| values)
    });
    bench("offsets clone + into_iter_owned len sum", || {
        black_box(&offsets)
            .clone()
            .into_iter_owned()
            .map(|item| item.len())
            .sum::<usize>()
    });
    bench("offsets clone + into_iter_owned collect", || {
        black_box(&offsets)
            .clone()
            .into_iter_owned()
            .collect::<Vec<Vec<u32>>>()
    });
}
