//! `Tree`'s digest is sensitive to its input.

mod common;
use common::{
    digest,
    digest_bytes,
};

#[test]
fn different_entry_names_produce_different_tree_digests() {
    let blob = digest_bytes(b"content");
    let a = ply::Tree::new([("a".to_string(), ply::Node::Blob(blob))], []);
    let b = ply::Tree::new([("b".to_string(), ply::Node::Blob(blob))], []);
    assert_ne!(digest(&a), digest(&b));
}

#[test]
fn a_blob_and_a_tree_with_the_same_inner_digest_have_different_tree_digests() {
    // A blob and a nested tree that happen to wrap the same inner
    // digest must not collide.
    let inner = digest_bytes(b"same");
    let a = ply::Tree::new([("x".to_string(), ply::Node::Blob(inner))], []);
    let b = ply::Tree::new([("x".to_string(), ply::Node::Tree(inner))], []);
    assert_ne!(digest(&a), digest(&b));
}

#[test]
fn different_nested_tree_content_produces_different_tree_digests() {
    let inner_a = ply::Tree::new([("f".to_string(), ply::Node::Blob(digest_bytes(b"a")))], []);
    let inner_b = ply::Tree::new([("f".to_string(), ply::Node::Blob(digest_bytes(b"b")))], []);

    let a = ply::Tree::new([("dir".to_string(), ply::Node::Tree(digest(&inner_a)))], []);
    let b = ply::Tree::new([("dir".to_string(), ply::Node::Tree(digest(&inner_b)))], []);
    assert_ne!(digest(&a), digest(&b));
}

#[test]
fn different_interns_produce_different_tree_digests() {
    let a = ply::Tree::new([], [digest_bytes(b"intern-a")]);
    let b = ply::Tree::new([], [digest_bytes(b"intern-b")]);
    assert_ne!(digest(&a), digest(&b));
}
