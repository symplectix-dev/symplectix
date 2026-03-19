//! Benchmarks for wavelet matrix operations.
use std::hint::black_box;

use compacts::ops::*;
use compacts::{
    BitArray,
    WaveletMatrix,
};
use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use lazy_static::lazy_static;
use rand::prelude::*;

type BitMap = compacts::BitMap<[u64; 1024]>;

macro_rules! generate {
    ($rng:expr, $len:expr, $tab:expr) => {{
        let mut build = vec![0; $len];
        for i in 0..$len {
            build[i] = $tab[$rng.random_range(0..$tab.len())];
        }
        build
    }};
    ($rng:expr, $len:expr) => {{
        let mut build = vec![0u32; $len];
        for i in 0..$len {
            build[i] = $rng.random_range(0..$len as u32);
        }
        build
    }};
}

const LENGTH: usize = 100_000_000;

lazy_static! {
    static ref T1: Vec<u32> = generate!(rand::rng(), 1000);
    static ref S0: Vec<u32> = generate!(rand::rng(), LENGTH);
    static ref S1: Vec<u32> = generate!(rand::rng(), 100_000_000, T1);
}

mod wm_vec {
    use super::*;

    lazy_static! {
        pub static ref W0: WaveletMatrix<u32, BitArray<u64>> = {
            let mut vec = S0.clone();
            WaveletMatrix::from(&mut vec[..])
        };
        pub static ref W1: WaveletMatrix<u32, BitArray<u64>> = {
            let mut vec = S1.clone();
            WaveletMatrix::from(&mut vec[..])
        };
    }
}

mod wm_map {
    use super::*;

    lazy_static! {
        pub static ref W0: WaveletMatrix<u32, BitMap> = {
            let mut vec = S0.clone();
            WaveletMatrix::from(&mut vec[..])
        };
        pub static ref W1: WaveletMatrix<u32, BitMap> = {
            let mut vec = S1.clone();
            WaveletMatrix::from(&mut vec[..])
        };
    }
}

fn wm_vec(c: &mut Criterion) {
    use wm_vec::{
        W0,
        W1,
    };
    let mut g = c.benchmark_group("wm_vec");

    g.bench_function("rank5", |b| {
        b.iter(|| black_box(W0.rank(&5, ..rand::rng().random_range(0..W0.size()))))
    });
    g.bench_function("rank5_all", |b| {
        b.iter(|| black_box(W0.view(0..rand::rng().random_range(0..W0.size())).counts(&5)))
    });
    g.bench_function("rank7", |b| {
        b.iter(|| black_box(W0.rank(&7, ..rand::rng().random_range(0..W0.size()))))
    });

    let c5 = W0.rank(&5, ..W0.size());
    g.bench_function("select5", |b| b.iter(|| black_box(W0.select(&5, c5 / 2))));

    let c7 = W0.rank(&7, ..W0.size());
    g.bench_function("select7", |b| b.iter(|| black_box(W0.select(&7, c7 / 2))));

    g.bench_function("quantile", |b| {
        b.iter(|| {
            black_box(W0.view(2_000_000..14_000_000).quantile(rand::rng().random_range(0..1000)))
        })
    });

    g.bench_function("topk", |b| {
        let m = rand::rng().random_range(0..2_000_000);
        let n = rand::rng().random_range(0..7_000_000);
        b.iter(|| black_box(W1.view(m..m + n).topk(1000)))
    });
    g.bench_function("mink", |b| {
        let m = rand::rng().random_range(0..2_000_000);
        let n = rand::rng().random_range(0..7_000_000);
        b.iter(|| black_box(W1.view(m..m + n).mink(1000)))
    });
    g.bench_function("maxk", |b| {
        let m = rand::rng().random_range(0..2_000_000);
        let n = rand::rng().random_range(0..7_000_000);
        b.iter(|| black_box(W1.view(m..m + n).maxk(1000)))
    });

    g.finish();
}

fn wm_map(c: &mut Criterion) {
    use wm_map::{
        W0,
        W1,
    };
    let mut g = c.benchmark_group("wm_map");

    g.bench_function("rank5", |b| {
        b.iter(|| black_box(W0.rank(&5, ..rand::rng().random_range(0..W0.size()))))
    });
    g.bench_function("rank5_all", |b| {
        b.iter(|| black_box(W0.view(0..rand::rng().random_range(0..W0.size())).counts(&5)))
    });
    g.bench_function("rank7", |b| {
        b.iter(|| black_box(W0.rank(&7, ..rand::rng().random_range(0..W0.size()))))
    });

    let c5 = W0.rank(&5, ..W0.size());
    g.bench_function("select5", |b| b.iter(|| black_box(W0.select(&5, c5 / 2))));

    let c7 = W0.rank(&7, ..W0.size());
    g.bench_function("select7", |b| b.iter(|| black_box(W0.select(&7, c7 / 2))));

    g.bench_function("quantile", |b| {
        b.iter(|| {
            black_box(W0.view(2_000_000..14_000_000).quantile(rand::rng().random_range(0..1000)))
        })
    });

    g.bench_function("topk", |b| {
        let m = rand::rng().random_range(0..2_000_000);
        let n = rand::rng().random_range(0..7_000_000);
        b.iter(|| black_box(W1.view(m..m + n).topk(1000)))
    });
    g.bench_function("mink", |b| {
        let m = rand::rng().random_range(0..2_000_000);
        let n = rand::rng().random_range(0..7_000_000);
        b.iter(|| black_box(W1.view(m..m + n).mink(1000)))
    });
    g.bench_function("maxk", |b| {
        let m = rand::rng().random_range(0..2_000_000);
        let n = rand::rng().random_range(0..7_000_000);
        b.iter(|| black_box(W1.view(m..m + n).maxk(1000)))
    });

    g.finish();
}

/// Build benchmarks are slow and excluded from the default run.
/// Run explicitly with: cargo bench --bench wm -- wm_build
fn wm_build(c: &mut Criterion) {
    let mut g = c.benchmark_group("wm_build");

    g.bench_function("wm_vec", |b| {
        let mut vec = S0.clone();
        b.iter(|| black_box(WaveletMatrix::<u32, BitArray<u64>>::from(vec.as_mut_slice())))
    });
    g.bench_function("wm_map", |b| {
        let mut vec = S0.clone();
        b.iter(|| black_box(WaveletMatrix::<u32, BitMap>::from(vec.as_mut_slice())))
    });

    g.finish();
}

criterion_group!(wm_vec_benches, wm_vec);
criterion_group!(wm_map_benches, wm_map);
criterion_group!(
    name = wm_build_benches;
    config = Criterion::default().sample_size(10);
    targets = wm_build
);
criterion_main!(wm_vec_benches, wm_map_benches);
