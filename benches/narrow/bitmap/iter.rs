use criterion::{BenchmarkId, Criterion, Throughput};
use narrow::{bitmap::Bitmap, buffer::BoxBuffer, collection::Collection};
use rand::{Rng, SeedableRng, prelude::SmallRng};
use std::time::Duration;

pub(super) fn bench(c: &mut Criterion) {
    {
        let mut group = c.benchmark_group("Bitmap::from_iter");
        group.warm_up_time(Duration::from_millis(100));
        group.measurement_time(Duration::from_secs(1));

        let mut rng = SmallRng::seed_from_u64(1234);

        for size in [8192] {
            for null_fraction in [0.5] {
                let input = (0..size)
                    .map(|_| rng.random_bool(1. - null_fraction))
                    .collect::<Vec<_>>();
                group.throughput(Throughput::Elements(size as u64));
                group.bench_with_input(
                    BenchmarkId::new("narrow", format!("{size}/{null_fraction}")),
                    &input,
                    |b, input| b.iter(|| Bitmap::<BoxBuffer>::from_iter(input)),
                );
            }
        }
    }

    {
        let mut group = c.benchmark_group("Bitmap::into_iter");
        let mut rng = SmallRng::seed_from_u64(1234);

        for size in [8192] {
            for null_fraction in [0.5] {
                let input = (0..size)
                    .map(|_| rng.random_bool(1. - null_fraction))
                    .collect::<Vec<_>>();
                let narrow_bitmap = Bitmap::<BoxBuffer>::from_iter(&input);
                group.throughput(Throughput::Elements(size as u64));
                group.bench_with_input(
                    BenchmarkId::new("narrow", format!("{size}/{null_fraction}")),
                    &(),
                    |b, _| b.iter(|| Vec::<bool>::from_iter(narrow_bitmap.iter_views())),
                );
            }
        }
    }
}
