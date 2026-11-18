//! Dependency-free benchmarks for variable-length (offset) collections.
//!
//! Run with `cargo bench --bench offsets`.

mod common;

use std::hint::black_box;

use common::bench;
use narrow::{
    collection::{Collection, owned::IntoOwned},
    length::Length,
    offset::Offsets,
};

/// Number of variable-length items in each benchmarked collection.
const ITEMS: usize = 1 << 16;

/// Number of items in each benchmarked collection of long items.
const LONG_ITEMS: usize = 1 << 12;

/// Number of items in each benchmarked collection of fixed-length items.
const WIDE_ITEMS: usize = 1 << 10;

/// Length of each item in the collection of fixed-length items.
const WIDE_ITEM_LEN: usize = 256;

/// Deterministic, irregular item length between 0 and 15.
fn item_len(index: usize) -> usize {
    index.wrapping_mul(0x9e37_79b9) >> 11 & 0xf
}

/// Deterministic, irregular item length between 64 and 191.
fn long_item_len(index: usize) -> usize {
    64 + (index.wrapping_mul(0x9e37_79b9) >> 11 & 0x7f)
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
    let long = (0..LONG_ITEMS)
        .map(|index| {
            (0..long_item_len(index))
                .map(|value| u32::try_from(value).expect("small"))
                .collect::<Vec<_>>()
        })
        .collect::<Offsets<Vec<u32>>>();
    let wide_lists = (0..WIDE_ITEMS)
        .map(|index| {
            (0..WIDE_ITEM_LEN)
                .map(|value| u32::try_from(index ^ value).expect("small"))
                .collect()
        })
        .collect::<Vec<Vec<u32>>>();
    let wide = wide_lists.iter().cloned().collect::<Offsets<Vec<u32>>>();
    let nullable = lists
        .iter()
        .enumerate()
        .map(|(index, list)| {
            list.iter()
                .map(|&value| (index % 3 != 0).then_some(value))
                .collect()
        })
        .collect::<Offsets<Vec<Option<u32>>>>();
    let nested = (0..LONG_ITEMS)
        .map(|index| {
            (0..item_len(index))
                .map(|len| (0..u32::try_from(len).expect("small")).collect())
                .collect()
        })
        .collect::<Offsets<Offsets<Vec<u32>>>>();

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
            .map(|item| black_box(item).len())
            .sum::<usize>()
    });
    bench("offsets clone (baseline)", || black_box(&offsets).clone());
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
    bench("long offsets iter_views values sum", || {
        black_box(&long)
            .iter_views()
            .map(|item| item.iter_views().sum::<u32>())
            .sum::<u32>()
    });
    bench("long offsets iter_views values max", || {
        black_box(&long)
            .iter_views()
            .filter_map(|item| item.iter_views().max())
            .sum::<u32>()
    });
    bench("long offsets iter_views into_owned", || {
        black_box(&long)
            .iter_views()
            .map(IntoOwned::<Vec<u32>>::into_owned)
            .map(|item| black_box(item).len())
            .sum::<usize>()
    });
    bench("256-value slices to_vec (baseline)", || {
        black_box(&wide_lists)
            .iter()
            .map(|item| black_box(item.to_vec()).len())
            .sum::<usize>()
    });
    bench("256-value offsets iter_views into_owned", || {
        black_box(&wide)
            .iter_views()
            .map(IntoOwned::<Vec<u32>>::into_owned)
            .map(|item| black_box(item).len())
            .sum::<usize>()
    });
    bench("256-value into_iter_owned collect", || {
        black_box(&wide)
            .iter_views()
            .map(|item| black_box(item.into_iter_owned().collect::<Vec<u32>>()).len())
            .sum::<usize>()
    });
    bench("short into_iter_owned collect", || {
        black_box(&offsets)
            .iter_views()
            .map(|item| black_box(item.into_iter_owned().collect::<Vec<u32>>()).len())
            .sum::<usize>()
    });
    bench("nullable offsets iter_views values sum", || {
        black_box(&nullable)
            .iter_views()
            .map(|item| item.iter_views().flatten().fold(0_u32, u32::wrapping_add))
            .fold(0_u32, u32::wrapping_add)
    });
    bench("nullable offsets iter_views into_owned", || {
        black_box(&nullable)
            .iter_views()
            .map(IntoOwned::<Vec<Option<u32>>>::into_owned)
            .map(|item| black_box(item).len())
            .sum::<usize>()
    });
    bench("nullable clone + into_iter_owned collect", || {
        black_box(&nullable)
            .clone()
            .into_iter_owned()
            .collect::<Vec<Vec<Option<u32>>>>()
    });
    bench("nested offsets iter_views into_owned", || {
        black_box(&nested)
            .iter_views()
            .map(IntoOwned::<Vec<Vec<u32>>>::into_owned)
            .map(|item| black_box(item).len())
            .sum::<usize>()
    });
    bench("nested clone + into_iter_owned collect", || {
        black_box(&nested)
            .clone()
            .into_iter_owned()
            .collect::<Vec<Vec<Vec<u32>>>>()
    });
    bench("long clone + into_iter_owned collect", || {
        black_box(&long)
            .clone()
            .into_iter_owned()
            .collect::<Vec<Vec<u32>>>()
    });
}
