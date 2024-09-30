use arrow_array::{builder::PrimitiveBuilder, ArrowPrimitiveType, PrimitiveArray};
use criterion::{BenchmarkId, Criterion};
use narrow::{array::FixedSizePrimitiveArray, FixedSize};
use num_traits::{Bounded, NumCast};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::{ops::Rem, time::Duration};

pub fn bench<T: ArrowPrimitiveType>(c: &mut Criterion)
where
    <T as ArrowPrimitiveType>::Native: NumCast + Bounded + Rem + FixedSize,
{
    let mut group = c.benchmark_group("PrimitiveBuilder");
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_secs(1));

    let max: usize = num_traits::cast(T::Native::max_value()).unwrap();

    for size in [0, 4, 8, 16].map(|v| 1usize << v).into_iter() {
        let input = (0..size)
            .map(|v| num_traits::cast(v % max).unwrap())
            .collect::<Vec<T::Native>>();
        group.throughput(criterion::Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("Arrow", size), &input, |bencher, input| {
            bencher
                .iter(|| arrow_build_primitive_array_from_iterator::<T>(input.clone().into_iter()))
        });
        group.bench_with_input(
            BenchmarkId::new("Narrow", size),
            &input,
            |bencher, input| {
                bencher.iter(|| {
                    narrow_build_primitive_array_from_iterator::<T>(input.clone().into_iter())
                })
            },
        );
    }
}

fn arrow_build_primitive_array_from_iterator<T>(
    input: impl ExactSizeIterator<Item = T::Native>,
) -> PrimitiveArray<T>
where
    T: ArrowPrimitiveType,
{
    let mut builder: PrimitiveBuilder<T> = PrimitiveBuilder::with_capacity(input.len());
    builder.extend(input.into_iter().map(Some));
    builder.finish()
}

fn narrow_build_primitive_array_from_iterator<T>(
    input: impl Iterator<Item = T::Native>,
) -> FixedSizePrimitiveArray<T::Native, false>
where
    T: ArrowPrimitiveType,
    <T as ArrowPrimitiveType>::Native: FixedSize,
{
    input.into_iter().collect()
}

pub fn bench_nullable<T: ArrowPrimitiveType>(c: &mut Criterion)
where
    <T as ArrowPrimitiveType>::Native: NumCast + Bounded + Rem + FixedSize,
{
    let mut group = c.benchmark_group("NullablePrimitiveBuilder");
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_secs(1));

    let max: usize = num_traits::cast(T::Native::max_value()).unwrap();
    let mut rng = SmallRng::seed_from_u64(1337);

    for size in [0, 4, 8, 16].map(|v| 1usize << v).into_iter() {
        for null_fraction in [0., 0.5, 1.] {
            let input = (0..size)
                .map(|v| num_traits::cast(v % max).unwrap())
                .map(|v| rng.gen_bool(1. - null_fraction).then_some(v))
                .collect::<Vec<Option<T::Native>>>();
            group.throughput(criterion::Throughput::Elements(size as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("Arrow/{}/", null_fraction), size),
                &input,
                |bencher, input| {
                    bencher.iter(|| {
                        arrow_build_primitive_array_from_iterator_nullable::<T>(
                            input.clone().into_iter(),
                        )
                    })
                },
            );
            group.bench_with_input(
                BenchmarkId::new(format!("Narrow/{}/", null_fraction), size),
                &input,
                |bencher, input| {
                    bencher.iter(|| {
                        narrow_build_primitive_array_from_iterator_nullable::<T>(
                            input.clone().into_iter(),
                        )
                    })
                },
            );
        }
    }
}

fn arrow_build_primitive_array_from_iterator_nullable<T>(
    input: impl ExactSizeIterator<Item = Option<T::Native>>,
) -> PrimitiveArray<T>
where
    T: ArrowPrimitiveType,
{
    let mut builder: PrimitiveBuilder<T> = PrimitiveBuilder::with_capacity(input.len());
    builder.extend(input);
    builder.finish()
}

fn narrow_build_primitive_array_from_iterator_nullable<T>(
    input: impl Iterator<Item = Option<T::Native>>,
) -> FixedSizePrimitiveArray<T::Native, true>
where
    T: ArrowPrimitiveType,
    <T as ArrowPrimitiveType>::Native: FixedSize,
{
    input.into_iter().collect()
}
