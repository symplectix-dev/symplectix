//! Benchmarks for bit-vector primitives.

use std::hint::black_box;

use bitcomp_roaring::bit_set::BitSet;
use bits::{
    Bits,
    BitsMut,
    Block,
    Buf,
};
use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use rand::prelude::*;

const NBITS: u64 = 150_000;
const BOUND: u64 = 10_000_000;

type Uncompressed = Vec<Buf<[u64; 1024]>>;
type Roaring = BitSet<u64>;

fn random_bits() -> (Uncompressed, Roaring) {
    let mut rng = rand::rng();
    let mut uncomp = vec![Buf::empty(); bits::blocks(BOUND, Buf::<[u64; 1024]>::BITS)];
    let mut roaring = BitSet::new();
    for _ in 0..NBITS {
        let bit = rng.random_range(0..BOUND);
        roaring.insert(bit);
        uncomp.set1(bit);
    }
    (uncomp, roaring)
}

fn benchmarks(c: &mut Criterion) {
    let (uncomp, roaring) = random_bits();

    let mut group = c.benchmark_group("bitcomp/rank");
    let i = BOUND / 2;
    group.bench_function("uncompressed/rank1", |b| {
        b.iter(|| {
            let _ = black_box(uncomp.rank1(..i));
        })
    });
    group.bench_function("roaring/rank1", |b| {
        b.iter(|| {
            let _ = black_box(roaring.rank1(i));
        })
    });
    group.bench_function("uncompressed/rank0", |b| {
        b.iter(|| {
            let _ = black_box(uncomp.rank0(..i));
        })
    });
    group.bench_function("roaring/rank0", |b| {
        b.iter(|| {
            let _ = black_box(roaring.rank0(i));
        })
    });
    group.finish();

    let mut group = c.benchmark_group("bitcomp/select");
    let c1 = uncomp.count1() / 2;
    let c0 = uncomp.count0() / 2;
    group.bench_function("uncompressed/select1", |b| {
        b.iter(|| {
            let _ = black_box(uncomp.select1(c1));
        })
    });
    group.bench_function("roaring/select1", |b| {
        b.iter(|| {
            let _ = black_box(roaring.select1(c1));
        })
    });
    group.bench_function("uncompressed/select0", |b| {
        b.iter(|| {
            let _ = black_box(uncomp.select0(c0));
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
