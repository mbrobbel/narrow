// use std::mem;
use criterion::{BenchmarkId, Criterion, Throughput};
use narrow::{Int64Array, Take};
use rand::{prelude::StdRng, Rng, SeedableRng};

pub(super) fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Take");
    let mut rng = StdRng::seed_from_u64(1234);

    // for size in [1024, 2048, 4096].iter() {
    for size in [16777216] {
        for select_size in [16777216] {
            // for select_size in [128, 256, 512, 1024, 2048, 4096, 8192] {
            let input = (0..size)
                .into_iter()
                .map(|_| Some(rand::random::<i64>()))
                .collect::<Int64Array<true>>();
            let indices = (0..select_size)
                .into_iter()
                .map(|_| Some(rng.gen_range(0..size)))
                .collect::<Int64Array<true>>();

            // group.throughput(Throughput::Bytes(
            //     select_size * mem::size_of::<i64>() as u64,
            // ));
            // todo(mb): array string trait e.g. fixed_size_primitive<int64, true>
            group.throughput(Throughput::Elements(select_size as u64));
            group.bench_with_input(
                BenchmarkId::new("take", format!("{}/{}", size, select_size)),
                &input,
                |b, input| {
                    b.iter(|| {
                        input.take(&indices);
                    })
                },
            );
        }
    }
}
