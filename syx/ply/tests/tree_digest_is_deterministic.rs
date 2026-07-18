//! `Tree`'s digest is deterministic.

mod common;
use common::digest;

#[test]
fn hashing_an_empty_tree_twice_gives_the_same_digest() {
    assert_eq!(cas::digest(&ply::Tree::new([], [])), cas::digest(&ply::Tree::new([], [])));
}

#[test]
fn tree_digest_ignores_entry_build_order() {
    let a = ("a".to_string(), ply::Node::Blob(digest(b"a")));
    let b = ("b".to_string(), ply::Node::Blob(digest(b"b")));

    let forward = ply::Tree::new([a.clone(), b.clone()], []);
    let backward = ply::Tree::new([b, a], []);

    assert_eq!(cas::digest(&forward), cas::digest(&backward));
}

#[test]
fn tree_digest_ignores_intern_build_order() {
    let a = digest(b"a");
    let b = digest(b"b");
    assert_eq!(cas::digest(&ply::Tree::new([], [a, b])), cas::digest(&ply::Tree::new([], [b, a])));
}
