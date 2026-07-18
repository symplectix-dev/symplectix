//! `Tree`'s digest is deterministic.

mod common;
use common::{
    digest,
    digest_bytes,
};

#[test]
fn hashing_an_empty_tree_twice_gives_the_same_digest() {
    assert_eq!(digest(&ply::Tree::new([], [])), digest(&ply::Tree::new([], [])));
}

#[test]
fn tree_digest_ignores_entry_build_order() {
    let a = ("a".to_string(), ply::Node::Blob(digest_bytes(b"a")));
    let b = ("b".to_string(), ply::Node::Blob(digest_bytes(b"b")));

    let forward = ply::Tree::new([a.clone(), b.clone()], []);
    let backward = ply::Tree::new([b, a], []);

    assert_eq!(digest(&forward), digest(&backward));
}

#[test]
fn tree_digest_ignores_intern_build_order() {
    let a = digest_bytes(b"a");
    let b = digest_bytes(b"b");
    assert_eq!(digest(&ply::Tree::new([], [a, b])), digest(&ply::Tree::new([], [b, a])));
}
