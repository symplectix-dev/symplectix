//! Benchmark rrr where b = 15.

use std::hint::black_box;

use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use rand::prelude::*;

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitcomp_rrr");
    group.bench_function("encode", |b| {
        let mut rng = rand::rng();
        b.iter(|| {
            let _ = black_box(bitcomp_rrr::encode(rng.random()));
        })
    });
    group.bench_function("decode", |b| {
        let mut rng = rand::rng();
        let (c, o) = bitcomp_rrr::encode(rng.random());
        b.iter(|| {
            let _ = black_box(bitcomp_rrr::decode(c, o));
        })
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
