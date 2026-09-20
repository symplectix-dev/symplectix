//! `Tree`'s digest is deterministic.

mod common;
use common::{
    Node,
    Tree,
};

#[test]
fn hashing_an_empty_tree_twice_gives_the_same_digest() {
    assert_eq!(cas_testing::digest(&Tree::new([], [])), cas_testing::digest(&Tree::new([], [])));
}

#[test]
fn tree_digest_ignores_entry_build_order() {
    let a = ("a".to_string(), Node::Blob(cas_testing::digest_bytes(b"a")));
    let b = ("b".to_string(), Node::Blob(cas_testing::digest_bytes(b"b")));

    let forward = Tree::new([a.clone(), b.clone()], []);
    let backward = Tree::new([b, a], []);

    assert_eq!(cas_testing::digest(&forward), cas_testing::digest(&backward));
}

#[test]
fn tree_digest_ignores_intern_build_order() {
    let a = cas_testing::digest_bytes(b"a");
    let b = cas_testing::digest_bytes(b"b");
    assert_eq!(
        cas_testing::digest(&Tree::new([], [a, b])),
        cas_testing::digest(&Tree::new([], [b, a]))
    );
}
