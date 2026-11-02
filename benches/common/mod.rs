//! A dependency-free benchmark harness shared by the benchmarks.

use std::{
    hint::black_box,
    iter,
    time::{Duration, Instant},
};

/// Minimum measured time per sample.
const SAMPLE_TIME: Duration = Duration::from_millis(200);
/// Number of samples; the fastest one is reported.
const SAMPLES: usize = 5;

/// Measures `f` and prints its fastest time per call under `name`.
pub fn bench<T>(name: &str, mut f: impl FnMut() -> T) {
    // `cargo bench` passes `--bench`; under `cargo test` only smoke-test once.
    if !std::env::args().any(|arg| arg == "--bench") {
        black_box(f());
        println!("{name:<40} ok");
        return;
    }

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
