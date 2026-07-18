//! `Digest::hex` formatting.

mod common;
use common::digest_bytes;

#[test]
fn hex_with_depth_zero_matches_plain_hex_formatting() {
    let d = digest_bytes(b"hello");
    assert_eq!(d.hex(0), format!("{d:x}"));
}

#[test]
fn hex_output_is_lowercase_and_64_characters_long() {
    let d = digest_bytes(b"hello");
    let hex = d.hex(0);
    assert_eq!(hex.len(), 64);
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
}

#[test]
fn hex_with_depth_splits_leading_byte_pairs() {
    let d = digest_bytes(b"hello");
    let plain = format!("{d:x}");
    let want = format!("{}/{}/{}/{}", &plain[0..2], &plain[2..4], &plain[4..6], &plain[6..]);
    assert_eq!(d.hex(3), want);
}

#[test]
#[should_panic]
fn hex_panics_when_depth_is_32_or_more() {
    digest_bytes(b"hello").hex(32);
}
