//! Benchmark roaring partitioning.

use std::hint::black_box;
use std::ops::Range;

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

type BufVec = Vec<Buf<[u64; 1024]>>;
type Roaring = BitSet<u64>;

fn gen_bits(r: Range<u64>) -> (BufVec, Roaring) {
    let mut rng = rand::rng();
    let mut bufvec = vec![Buf::empty(); bits::blocks(BOUND, Buf::<[u64; 1024]>::BITS)];
    let mut roaring = BitSet::new();
    for _ in 0..NBITS {
        let bit = rng.random_range(r.clone());
        roaring.insert(bit);
        bufvec.set1(bit);
    }
    (bufvec, roaring)
}

fn benchmarks(c: &mut Criterion) {
    let (bv, roaring_bv) = gen_bits(0..BOUND);

    let mut group = c.benchmark_group("rank1");
    let i = 1 << 20;
    group.bench_function("BufVec", |b| {
        b.iter(|| {
            let _ = black_box(bv.rank1(..i));
        })
    });
    group.bench_function("Roaring", |b| {
        b.iter(|| {
            let _ = black_box(roaring_bv.rank1(i));
        })
    });
    group.finish();

    let mut group = c.benchmark_group("rank0");
    group.bench_function("BufVec", |b| {
        b.iter(|| {
            let _ = black_box(bv.rank0(..i));
        })
    });
    group.bench_function("Roaring", |b| {
        b.iter(|| {
            let _ = black_box(roaring_bv.rank0(i));
        })
    });
    group.finish();

    let n = 10000;
    let mut group = c.benchmark_group("select1");
    group.bench_function("BufVec", |b| {
        b.iter(|| {
            let _ = black_box(bv.select1(n));
        })
    });
    group.bench_function("Roaring", |b| {
        b.iter(|| {
            let _ = black_box(roaring_bv.select1(n));
        })
    });
    group.finish();

    let mut group = c.benchmark_group("select0");
    group.bench_function("BufVec", |b| {
        b.iter(|| {
            let _ = black_box(bv.select0(n));
        })
    });
    group.bench_function("Roaring", |b| {
        b.iter(|| {
            let _ = black_box(roaring_bv.select0(n));
        })
    });
    group.finish();
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
