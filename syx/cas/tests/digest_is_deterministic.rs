//! Digest is deterministic.

mod support;
use support::{
    Example,
    digest_bytes,
    digest_parts,
};

#[test]
fn hashing_the_same_bytes_twice_gives_the_same_digest() {
    assert_eq!(digest_bytes(b"a".as_slice()), digest_bytes(b"a".as_slice()));
}

#[test]
fn hashing_the_same_struct_twice_gives_the_same_digest() {
    let a = Example { name: "foo".to_string(), count: 1 };
    let b = Example { name: "foo".to_string(), count: 1 };
    assert_eq!(cas::digest(&a), cas::digest(&b));
}

#[test]
fn chaining_part_gives_the_same_digest_as_parts_over_same_sequence() {
    let mut h = cas::Hasher::new();
    h.part(b"a").part(b"b");
    assert_eq!(h.digest(), digest_parts([b"a".as_slice(), b"b".as_slice()]));
}

#[test]
fn hashing_no_parts_twice_gives_the_same_digest() {
    assert_eq!(digest_parts(Vec::<&[u8]>::new()), digest_parts(Vec::<&[u8]>::new()));
}
