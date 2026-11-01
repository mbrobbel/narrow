//! Dependency-free benchmarks for bitmap counting and iteration.
//!
//! Run with `cargo bench --bench bitmap`.

use std::{
    hint::black_box,
    iter,
    time::{Duration, Instant},
};

use narrow::{
    bitmap::{Bitmap, ValidityBitmap},
    buffer::VecBuffer,
    collection::{Collection, CollectionRealloc},
    validity::Validity,
};

/// Number of logical bits in each benchmarked bitmap.
const BITS: usize = 1 << 20;
/// A bit offset that is not byte aligned.
const OFFSET: usize = 3;
/// Minimum measured time per sample.
const SAMPLE_TIME: Duration = Duration::from_millis(200);
/// Number of samples; the fastest one is reported.
const SAMPLES: usize = 5;

fn bench<T>(name: &str, mut f: impl FnMut() -> T) {
    // Runs `f` the given number of times and returns the elapsed time.
    let mut time = |iterations: u32| {
        let start = Instant::now();
        (0..iterations).for_each(|_| {
            black_box(f());
        });
        start.elapsed()
    };

    // Warm up and find an iteration count that fills a sample.
    let iterations = iter::successors(Some(1_u32), |iterations| iterations.checked_mul(2))
        .find(|&iterations| time(iterations) >= SAMPLE_TIME / 4)
        .unwrap_or(u32::MAX)
        .saturating_mul(4);

    let best = (0..SAMPLES)
        .map(|_| time(iterations) / iterations)
        .min()
        .expect("at least one sample");
    println!("{name:<40} {:>12.3} µs/iter", best.as_secs_f64() * 1e6);
}

/// Deterministic, irregular bit pattern with about two thirds of the bits set.
fn pattern(index: usize) -> bool {
    // The shift breaks up the multiplication's structure: without it, the
    // result's remainder modulo 3 follows directly from `index`'s.
    let hash = index.wrapping_mul(0x9e37_79b9);
    !(hash >> 11).is_multiple_of(3)
}

fn main() {
    let bytes: Vec<u8> = Bitmap::<VecBuffer>::from_iter((0..OFFSET + BITS).map(pattern))
        .into_parts()
        .0;
    let bitmap = Bitmap::<VecBuffer>::try_from_parts(bytes, BITS, OFFSET).expect("valid parts");
    let validity = (0..BITS)
        .map(|index| pattern(index).then_some(0_u8))
        .collect::<Validity<Vec<u8>>>();

    let bools: Vec<bool> = (0..BITS).map(pattern).collect();
    let all_valid = (0..BITS).map(|_| Some(0_u8)).collect::<Validity<Vec<u8>>>();

    // Construction.
    bench("bitmap from_iter", || {
        Bitmap::<VecBuffer>::from_iter(black_box(&bools))
    });
    bench("bitmap extend (unaligned)", || {
        let mut bitmap = Bitmap::<VecBuffer>::from_iter([true; 3]);
        bitmap.extend(black_box(&bools));
        bitmap
    });
    bench("bitmap extend (overwrite after truncate)", || {
        let mut bitmap = black_box(&bitmap).clone();
        bitmap.truncate(3);
        bitmap.extend(black_box(&bools));
        bitmap
    });
    bench("validity from_iter", || {
        black_box(&bools)
            .iter()
            .map(|valid| valid.then_some(0_u8))
            .collect::<Validity<Vec<u8>>>()
    });
    bench("validity extend", || {
        let mut validity = Validity::<Vec<u8>>::default();
        validity.extend(black_box(&bools).iter().map(|valid| valid.then_some(0_u8)));
        validity
    });

    // Counting and predicates.
    bench("validity null_count", || black_box(&validity).null_count());
    bench("validity any_null (all valid)", || {
        black_box(&all_valid).any_null()
    });
    bench("validity all_valid (all valid)", || {
        black_box(&all_valid).all_valid()
    });
    bench("validity all_null (mixed)", || {
        black_box(&validity).all_null()
    });

    // Iteration.
    bench("borrowed iter count true", || {
        black_box(&bitmap).into_iter().filter(|bit| *bit).count()
    });
    bench("borrowed iter collect Vec<bool>", || {
        black_box(&bitmap).into_iter().collect::<Vec<_>>()
    });
    bench("borrowed iter nth(last)", || {
        black_box(&bitmap).into_iter().nth(BITS - 1)
    });
    bench("owned clone (baseline)", || black_box(&bitmap).clone());
    bench("owned clone + iter count true", || {
        black_box(&bitmap)
            .clone()
            .into_iter()
            .filter(|bit| *bit)
            .count()
    });
    bench("owned clone + iter nth(last)", || {
        black_box(&bitmap).clone().into_iter().nth(BITS - 1)
    });
    bench("validity iter count valid", || {
        black_box(&validity).iter_views().flatten().count()
    });
}
