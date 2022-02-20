use arrow2::bitmap::MutableBitmap;
use bitvec::prelude::BitVec;
use criterion::{BenchmarkId, Criterion, Throughput};
use narrow::Bitmap;
use rand::{prelude::StdRng, Rng, SeedableRng};

pub(super) fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bitmap::from_iter");
    let mut rng = StdRng::seed_from_u64(1234);

    // for size in [16777216] {
    for size in [12345] {
        // for size in [1024, 2048, 4096] {
        for null_fraction in [0., 0.1, 0.5, 0.9, 1.] {
            let input = (0..size)
                .into_iter()
                .map(|_| rng.gen_bool(1. - null_fraction))
                .collect::<Vec<_>>();
            group.throughput(Throughput::Bytes(size as u64));
            // group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(
                BenchmarkId::new("narrow", format!("{}/{}", size, null_fraction)),
                &input,
                |b, input| b.iter(|| Bitmap::<Vec<u8>>::from_iter(input)),
            );
            group.bench_with_input(
                BenchmarkId::new("arrow2", format!("{}/{}", size, null_fraction)),
                &input,
                |b, input| b.iter(|| MutableBitmap::from_iter(input.iter().copied())),
            );
            group.bench_with_input(
                BenchmarkId::new("bitvec", format!("{}/{}", size, null_fraction)),
                &input,
                |b, input| b.iter(|| BitVec::<u8>::from_iter(input)),
            );
        }
    }
}
