use criterion::{criterion_group, criterion_main, Criterion};

fn bench_consensus_placeholder(c: &mut Criterion) {
    c.bench_function("consensus_placeholder", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, bench_consensus_placeholder);
criterion_main!(benches);