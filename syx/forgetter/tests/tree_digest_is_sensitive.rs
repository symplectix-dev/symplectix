//! `Tree`'s digest is sensitive to its input.

mod common;
use common::{
    Node,
    Tree,
};

#[test]
fn different_entry_names_produce_different_tree_digests() {
    let blob = cas_testing::digest_bytes(b"content");
    let a = Tree::new([("a".to_string(), Node::Blob(blob))], []);
    let b = Tree::new([("b".to_string(), Node::Blob(blob))], []);
    assert_ne!(cas_testing::digest(&a), cas_testing::digest(&b));
}

#[test]
fn a_blob_and_a_tree_with_the_same_inner_digest_have_different_tree_digests() {
    // A blob and a nested tree that happen to wrap the same inner
    // digest must not collide.
    let inner = cas_testing::digest_bytes(b"same");
    let a = Tree::new([("x".to_string(), Node::Blob(inner))], []);
    let b = Tree::new([("x".to_string(), Node::Tree(inner))], []);
    assert_ne!(cas_testing::digest(&a), cas_testing::digest(&b));
}

#[test]
fn different_nested_tree_content_produces_different_tree_digests() {
    let inner_a = Tree::new([("f".to_string(), Node::Blob(cas_testing::digest_bytes(b"a")))], []);
    let inner_b = Tree::new([("f".to_string(), Node::Blob(cas_testing::digest_bytes(b"b")))], []);

    let a = Tree::new([("dir".to_string(), Node::Tree(cas_testing::digest(&inner_a)))], []);
    let b = Tree::new([("dir".to_string(), Node::Tree(cas_testing::digest(&inner_b)))], []);
    assert_ne!(cas_testing::digest(&a), cas_testing::digest(&b));
}

#[test]
fn different_interns_produce_different_tree_digests() {
    let a = Tree::new([], [cas_testing::digest_bytes(b"intern-a")]);
    let b = Tree::new([], [cas_testing::digest_bytes(b"intern-b")]);
    assert_ne!(cas_testing::digest(&a), cas_testing::digest(&b));
}
