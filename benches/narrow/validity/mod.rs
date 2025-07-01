use criterion::Criterion;

mod iter;

pub(super) fn bench(c: &mut Criterion) {
    iter::bench(c);
}
