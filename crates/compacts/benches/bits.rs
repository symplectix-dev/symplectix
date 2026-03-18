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
use lazy_static::lazy_static;
use rand::prelude::*;

macro_rules! generate {
    (Vec; $rng:expr, $nbits:expr, $bound:expr) => {{
        let mut build = compacts::bits::sized($bound);
        for _ in 0..$nbits {
            build.put1($rng.random_range(0..$bound));
        }
        build
    }};
    (Pop; $rng:expr, $nbits:expr, $bound:expr) => {{
        let mut build = Pop::new($bound);
        for _ in 0..$nbits {
            build.put1($rng.random_range(0..$bound));
        }
        build
    }};
    (BitMap; $rng:expr, $nbits:expr, $bound:expr) => {{
        let mut build = BitMap::none($bound);
        for _ in 0..$nbits {
            build.put1($rng.random_range(0..$bound));
        }
        build
    }};
}

const BOUND: usize = 10_000_000;

lazy_static! {
    static ref NBITS: usize = BOUND / rand::rng().random_range(1..100);
    static ref V0: Vec<u64> = generate!(Vec; rand::rng(), *NBITS, BOUND);
    static ref V1: Vec<u64> = generate!(Vec; rand::rng(), *NBITS, BOUND);
    static ref V2: Vec<u64> = generate!(Vec; rand::rng(), *NBITS, BOUND);
    static ref P0: Pop<u64> = generate!(Pop; rand::rng(), *NBITS, BOUND);
    static ref P1: Pop<u64> = generate!(Pop; rand::rng(), *NBITS, BOUND);
    static ref P2: Pop<u64> = generate!(Pop; rand::rng(), *NBITS, BOUND);
    static ref M0: BitMap<[u64; 1024]> = generate!(BitMap; rand::rng(), *NBITS, BOUND);
    static ref M1: BitMap<[u64; 1024]> = generate!(BitMap; rand::rng(), *NBITS, BOUND);
    static ref M2: BitMap<[u64; 1024]> = generate!(BitMap; rand::rng(), *NBITS, BOUND);
    static ref A0: BitArray<u64> = BitArray::from(V0.clone());
    static ref A1: BitArray<u64> = BitArray::from(V1.clone());
    static ref A2: BitArray<u64> = BitArray::from(V2.clone());
}

fn bit_vec(c: &mut Criterion) {
    let mut g = c.benchmark_group("bit_vec");

    let cap = V0.size() - 1;
    g.bench_function("bit", |b| b.iter(|| black_box(V0.bit(rand::rng().random_range(0..cap)))));

    g.bench_function("put1", |b| {
        let mut v0 = V0.clone();
        let cap = v0.size() - 1;
        b.iter(|| v0.put1(rand::rng().random_range(0..cap)))
    });

    g.finish();
}

fn pop_vec(c: &mut Criterion) {
    let mut g = c.benchmark_group("pop_vec");

    g.bench_function("put1", |b| {
        let mut p0 = P0.clone();
        let cap = p0.len() - 1;
        b.iter(|| p0.put1(rand::rng().random_range(0..cap)))
    });

    g.finish();
}

fn bit_map(c: &mut Criterion) {
    let mut g = c.benchmark_group("bit_map");

    let cap = M0.size() - 1;
    g.bench_function("bit", |b| b.iter(|| black_box(M0.bit(rand::rng().random_range(0..cap)))));

    g.bench_function("put1", |b| {
        let mut m0 = M0.clone();
        let cap = m0.size() - 1;
        b.iter(|| m0.put1(rand::rng().random_range(0..cap)))
    });

    g.finish();
}

fn rank(c: &mut Criterion) {
    let mut g = c.benchmark_group("rank");

    g.bench_function("BitSlice", |b| {
        b.iter(|| black_box(V0.rank1(..rand::rng().random_range(0..V0.size()))))
    });
    g.bench_function("BitArray", |b| {
        b.iter(|| black_box(A0.rank1(..rand::rng().random_range(0..A0.size()))))
    });
    g.bench_function("BitMap", |b| {
        b.iter(|| black_box(M0.rank1(..rand::rng().random_range(0..M0.size()))))
    });
    g.bench_function("PopVec", |b| {
        b.iter(|| black_box(P0.rank1(..rand::rng().random_range(0..P0.len()))))
    });

    g.finish();
}

fn select(c: &mut Criterion) {
    let mut g = c.benchmark_group("select");

    let cap_v = V0.count1() - 1;
    g.bench_function("BitSlice", |b| {
        b.iter(|| black_box(V0.select1(rand::rng().random_range(0..cap_v))))
    });

    let cap_a = A0.count1() - 1;
    g.bench_function("BitArray", |b| {
        b.iter(|| black_box(A0.select1(rand::rng().random_range(0..cap_a))))
    });

    let cap_m = M0.count1() - 1;
    g.bench_function("BitMap", |b| {
        b.iter(|| black_box(M0.select1(rand::rng().random_range(0..cap_m))))
    });

    let cap_p = P0.count1() - 1;
    g.bench_function("PopVec", |b| {
        b.iter(|| black_box(P0.select1(rand::rng().random_range(0..cap_p))))
    });

    g.finish();
}

criterion_group!(bit_vec_benches, bit_vec);
criterion_group!(pop_vec_benches, pop_vec);
criterion_group!(bit_map_benches, bit_map);
criterion_group!(rank_benches, rank);
criterion_group!(select_benches, select);
criterion_main!(bit_vec_benches, pop_vec_benches, bit_map_benches, rank_benches, select_benches);
