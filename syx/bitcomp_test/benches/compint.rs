//! Benchmarking integer compression methods.

use std::hint::black_box;

use bitpacking::{
    BitPacker,
    BitPacker4x,
};
use criterion::{
    Criterion,
    Throughput,
    criterion_group,
    criterion_main,
};

fn benchmark(c: &mut Criterion) {
    {
        let mut group = c.benchmark_group("compint_scalar");
        group.bench_function("varint", |b| {
            let mut buf = vec![0u8; varint::MAX_VARINT_LEN32];
            let mut n = 0;
            b.iter(|| {
                n += black_box(varint::encode(&mut buf, &20));
            })
        });

        group.bench_function("streamvbyte", |b| {
            let mut buf = vec![0u8; varint::MAX_VARINT_LEN32];
            let mut n = 0;
            b.iter(|| {
                n += black_box(streamvbyte::encode_scalar(&mut buf, 20));
            })
        });
    }
    {
        let mut group = c.benchmark_group("compint_encode");
        let data: Vec<u32> = vec![
            7, 7, 7, 7, 11, 10, 15, 13, 6, 5, 3, 14, 5, 7, 15, 12, 1, 10, 8, 10, 12, 14, 13, 1, 10,
            1, 1, 10, 4, 15, 12, 1, 2, 0, 8, 5, 14, 5, 2, 4, 1, 6, 14, 13, 5, 10, 10, 1, 6, 4, 1,
            12, 1, 1, 5, 15, 15, 2, 8, 6, 4, 3, 10, 8, 8, 9, 2, 6, 10, 5, 7, 9, 0, 13, 15, 5, 13,
            10, 0, 2, 10, 14, 5, 9, 12, 8, 5, 10, 8, 8, 10, 5, 13, 8, 11, 14, 7, 14, 4, 2, 9, 12,
            14, 5, 15, 12, 0, 12, 13, 3, 13, 5, 4, 15, 9, 8, 9, 3, 3, 3, 1, 12, 0, 6, 11, 11, 12,
            4,
        ];

        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_function("streamvbyte", |b| {
            const VBYTE: usize = 256;
            let mut buf = vec![0u8; VBYTE * varint::MAX_VARINT_LEN32];
            // let mut ctl = vec![0u8; (data.len() + 3) / 4];
            let mut ctl = vec![0u8; data.len().div_ceil(4)];
            let mut n = 0;
            b.iter(|| {
                n += black_box(streamvbyte::encode(&mut buf, &mut ctl, &data));
            })
        });
        group.bench_function("bitpacker4x", |b| {
            // Detects if `SSE3` is available on the current computer.
            let bitpacker = BitPacker4x::new();
            // Computes the number of bits used for each integer in the blocks.
            let num_bits: u8 = bitpacker.num_bits(&data);
            assert_eq!(num_bits, 4);

            // The compressed vector will take exactly `num_bits * BitPacker4x::BLOCK_LEN / 8`.
            // But it is ok to have an output with a different len as long as it is larger
            // than this.
            let mut compressed = vec![0u8; 4 * BitPacker4x::BLOCK_LEN];

            let mut n = 0;
            b.iter(|| {
                n += black_box(bitpacker.compress(&data, &mut compressed[..], num_bits));
            })
        });
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
