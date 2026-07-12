//! Benchmarks for bit-vector primitives.

use std::hint::black_box;

use compacts::ops::*;
use compacts::{
    BitArray,
    BitMap,
    Pop,
};
use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use rand::prelude::*;

const NBITS: usize = 150_000;
const BOUND: usize = 10_000_000;

type Uncompressed = BitMap<[u64; 1024]>;
type PopVec = Pop<u64>;
type Poppy = BitArray<u64>;

fn random_bits() -> (Uncompressed, PopVec, Poppy) {
    let mut rng = rand::rng();
    let mut vec = compacts::bits::sized(BOUND);
    let mut uncomp = BitMap::none(BOUND);
    let mut pop = Pop::new(BOUND);
    for _ in 0..NBITS {
        let bit = rng.random_range(0..BOUND);
        vec.put1(bit);
        uncomp.put1(bit);
        pop.put1(bit);
    }
    (uncomp, pop, Poppy::from(vec))
}

fn benchmarks(c: &mut Criterion) {
    let (uncomp, popvec, poppy) = random_bits();

    let mut group = c.benchmark_group("compacts/rank");
    let i = BOUND / 2;
    group.bench_function("uncompressed/rank1", |b| {
        b.iter(|| {
            let _ = black_box(uncomp.rank1(..i));
        })
    });
    group.bench_function("poppy/rank1", |b| {
        b.iter(|| {
            let _ = black_box(poppy.rank1(..i));
        })
    });
    group.bench_function("popvec/rank1", |b| {
        b.iter(|| {
            let _ = black_box(popvec.rank1(..i));
        })
    });
    group.bench_function("uncompressed/rank0", |b| {
        b.iter(|| {
            let _ = black_box(uncomp.rank0(..i));
        })
    });
    group.bench_function("poppy/rank0", |b| {
        b.iter(|| {
            let _ = black_box(poppy.rank0(..i));
        })
    });
    group.bench_function("popvec/rank0", |b| {
        b.iter(|| {
            let _ = black_box(popvec.rank0(..i));
        })
    });
    group.finish();

    let mut group = c.benchmark_group("compacts/select");
    let c1 = uncomp.count1() / 2;
    let c0 = uncomp.count0() / 2;
    group.bench_function("uncompressed/select1", |b| {
        b.iter(|| {
            let _ = black_box(uncomp.select1(c1));
        })
    });
    group.bench_function("poppy/select1", |b| {
        b.iter(|| {
            let _ = black_box(poppy.select1(c1));
        })
    });
    group.bench_function("popvec/select1", |b| {
        b.iter(|| {
            let _ = black_box(popvec.select1(c1));
        })
    });
    group.bench_function("uncompressed/select0", |b| {
        b.iter(|| {
            let _ = black_box(uncomp.select0(c0));
        })
    });
    group.bench_function("poppy/select0", |b| {
        b.iter(|| {
            let _ = black_box(poppy.select0(c0));
        })
    });
    group.bench_function("popvec/select0", |b| {
        b.iter(|| {
            let _ = black_box(popvec.select0(c0));
        })
    });
    group.finish();
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
