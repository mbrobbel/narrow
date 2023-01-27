use criterion::{BenchmarkId, Criterion, Throughput};
use narrow::bitmap::Bitmap;
use rand::{prelude::SmallRng, Rng, SeedableRng};

pub(super) fn bench(c: &mut Criterion) {
    {
        let mut group = c.benchmark_group("Bitmap::from_iter");
        let mut rng = SmallRng::seed_from_u64(1234);

        for size in [12345] {
            for null_fraction in [0., 0.5, 1.] {
                let input = (0..size)
                    .into_iter()
                    .map(|_| rng.gen_bool(1. - null_fraction))
                    .collect::<Vec<_>>();
                group.throughput(Throughput::Elements(size as u64));
                group.bench_with_input(
                    BenchmarkId::new("narrow", format!("{size}/{null_fraction}")),
                    &input,
                    |b, input| b.iter(|| Bitmap::<Vec<u8>>::from_iter(input)),
                );
            }
        }
    }

    {
        let mut group = c.benchmark_group("Bitmap::into_iter");
        let mut rng = SmallRng::seed_from_u64(1234);

        for size in [12345] {
            for null_fraction in [0., 0.5, 1.] {
                let input = (0..size)
                    .into_iter()
                    .map(|_| rng.gen_bool(1. - null_fraction))
                    .collect::<Vec<_>>();
                let narrow_bitmap = Bitmap::<Vec<u8>>::from_iter(&input);
                group.throughput(Throughput::Elements(size as u64));
                group.bench_with_input(
                    BenchmarkId::new("narrow", format!("{size}/{null_fraction}")),
                    &narrow_bitmap,
                    |b, input| b.iter(|| Vec::<bool>::from_iter(input)),
                );
            }
        }
    }
}
