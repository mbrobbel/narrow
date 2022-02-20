use criterion::Criterion;

mod take;

pub(super) fn bench(c: &mut Criterion) {
    take::bench(c);
}
