//! Benchmarks for bit-vector primitives.

use std::hint::black_box;

use bitcomp_roaring::bit_set::BitSet;
use bits::{
    Bits,
    BitsMut,
    Block,
};
use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use rand::prelude::*;

const NBITS: u64 = 150_000;
const BOUND: u64 = 10_000_000;

type UncompVec = Vec<u64>;
type UncompPop = bitcomp_poppy::Pop<UncompVec>;
type Roaring = BitSet<u64>;

fn random_bits() -> (UncompVec, UncompPop, Roaring) {
    let mut rng = rand::rng();
    let mut uncomp_vec = vec![0; bits::blocks(BOUND, <u64 as Block>::BITS)];
    let mut uncomp_pop = UncompPop::new(BOUND);
    let mut roaring = BitSet::new();
    for _ in 0..NBITS {
        let bit = rng.random_range(0..BOUND);
        roaring.insert(bit);
        uncomp_vec.set1(bit);
        uncomp_pop.set1(bit);
    }
    (uncomp_vec, uncomp_pop, roaring)
}

fn benchmarks(c: &mut Criterion) {
    let (uncomp_vec, uncomp_pop, roaring) = random_bits();

    let mut group = c.benchmark_group("bitcomp/rank");
    let i = BOUND / 2;
    group.bench_function("uncomp_vec/rank1", |b| {
        b.iter(|| {
            let _ = black_box(uncomp_vec.rank1(..i));
        })
    });
    group.bench_function("uncomp_pop/rank1", |b| {
        b.iter(|| {
            let _ = black_box(uncomp_pop.rank1(..i));
        })
    });
    group.bench_function("roaring/rank1", |b| {
        b.iter(|| {
            let _ = black_box(roaring.rank1(i));
        })
    });
    group.bench_function("uncomp_vec/rank0", |b| {
        b.iter(|| {
            let _ = black_box(uncomp_vec.rank0(..i));
        })
    });
    group.bench_function("uncomp_pop/rank0", |b| {
        b.iter(|| {
            let _ = black_box(uncomp_pop.rank0(..i));
        })
    });
    group.bench_function("roaring/rank0", |b| {
        b.iter(|| {
            let _ = black_box(roaring.rank0(i));
        })
    });
    group.finish();

    let mut group = c.benchmark_group("bitcomp/select");
    let c1 = uncomp_vec.count1() / 2;
    let c0 = uncomp_vec.count0() / 2;
    group.bench_function("uncomp_vec/select1", |b| {
        b.iter(|| {
            let _ = black_box(uncomp_vec.select1(c1));
        })
    });
    group.bench_function("uncomp_pop/select1", |b| {
        b.iter(|| {
            let _ = black_box(uncomp_pop.select1(c1));
        })
    });
    group.bench_function("roaring/select1", |b| {
        b.iter(|| {
            let _ = black_box(roaring.select1(c1));
        })
    });
    group.bench_function("uncomp_vec/select0", |b| {
        b.iter(|| {
            let _ = black_box(uncomp_vec.select0(c0));
        })
    });
    group.bench_function("uncomp_pop/select0", |b| {
        b.iter(|| {
            let _ = black_box(uncomp_pop.select0(c0));
        })
    });
    group.bench_function("roaring/select0", |b| {
        b.iter(|| {
            let _ = black_box(roaring.select0(c0));
        })
    });
    group.finish();
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
