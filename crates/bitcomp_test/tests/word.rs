//! Tests for `Bits::word`:
//!
//! 1. Primitive `word<T>` used `mask!(i, i+n) = ((1 << n) - 1) << i`, which panics in debug (or
//!    silently returns 0 in release) when n == Self::BITS (e.g., n=64 for u64).
//!
//! 2. Slice `word<T>` has a final partial-block read guarded only by `e < self.len()`, but when the
//!    read ends exactly on a block boundary q==0, cur has already reached T::BITS (e.g., 128). The
//!    `<< cur` then panics for u128.

use bitcomp_core::Bits;
use quickcheck_macros::quickcheck;

#[quickcheck]
fn u8_word_full(v: u8) -> bool {
    v == v.word::<u8>(0, 8)
}
#[quickcheck]
fn u8_word_partial(v: u8) -> bool {
    v >> 4 == v.word::<u8>(4, 8)
}

#[quickcheck]
fn u16_word_full(v: u16) -> bool {
    v == v.word::<u16>(0, 16)
}
#[quickcheck]
fn u16_word_partial(v: u16) -> bool {
    v >> 8 == v.word::<u16>(8, 16)
}

#[quickcheck]
fn u32_word_full(v: u32) -> bool {
    v == v.word::<u32>(0, 32)
}
#[quickcheck]
fn u32_word_partial(v: u32) -> bool {
    v >> 16 == v.word::<u32>(16, 32)
}

#[quickcheck]
fn u64_word_full(v: u64) -> bool {
    v == v.word::<u64>(0, 64)
}
#[quickcheck]
fn u64_word_partial(v: u64) -> bool {
    v >> 32 == v.word::<u64>(32, 32)
}

#[test]
fn slice_word_u64_aligned() {
    let v = vec![0xAAAA_AAAA_AAAA_AAAAu64, 0xBBBB_BBBB_BBBB_BBBB];
    assert_eq!(v.word::<u64>(0, 64), 0xAAAA_AAAA_AAAA_AAAA);
    assert_eq!(v.word::<u64>(64, 64), 0xBBBB_BBBB_BBBB_BBBB);
}

#[test]
fn slice_word_u64_unaligned() {
    let v = vec![0xAAAA_AAAA_AAAA_AAAAu64, 0xBBBB_BBBB_BBBB_BBBB];
    assert_eq!(v.word::<u64>(48, 32), 0x0000_0000_BBBB_AAAA);
    assert_eq!(v.word::<u64>(32, 32), 0x0000_0000_AAAA_AAAA);
    assert_eq!(v.word::<u64>(32, 64), 0xBBBB_BBBB_AAAA_AAAA);
}

#[test]
fn slice_word_u128_aligned() {
    // Read 128 bits from a 64-bit-aligned offset: this is the primary regression case.
    // Previously: u64::word(0, 64) overflowed, AND when q==0 the slice impl shifted
    // by cur==128 (for u128), both causing panics.
    let v = vec![
        0x1111_1111_1111_1111u64,
        0x2222_2222_2222_2222u64,
        0x3333_3333_3333_3333u64,
        0x4444_4444_4444_4444u64,
    ];

    let w: u128 = v.word::<u128>(0, 128);
    assert_eq!(w, (v[1] as u128) << 64 | v[0] as u128);

    let w: u128 = v.word::<u128>(128, 128);
    assert_eq!(w, (v[3] as u128) << 64 | v[2] as u128);
}

#[test]
fn slice_word_u128_unaligned() {
    let v = vec![u64::MAX, 0u64, u64::MAX, 0u64];

    // Read 128 bits from the bit position 32:
    // The word u128 spans v[0], v[1] and v[2].
    assert_eq!(v.word::<u128>(32, 128), {
        let l = (v[0] >> 32) as u128;
        let m = (v[1] as u128) << 32;
        let h = ((v[2] & 0xFFFF_FFFF) as u128) << 96;
        l | m | h
    });
}

#[test]
fn slice_word_single_block() {
    // Read within a single u64 block (s == e path).
    let v = vec![0b_1010_1010u64, 0u64];
    assert_eq!(v.word::<u8>(0, 8), 0b_1010_1010);
    assert_eq!(v.word::<u8>(1, 4), 0b_0101);
    assert_eq!(v.word::<u8>(4, 4), 0b_1010);
}
