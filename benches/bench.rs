use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("ecc", |b| b.iter(|| true));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
