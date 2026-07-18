//! How `Tree` handles its `entries`.

mod common;
use common::digest;

#[test]
fn duplicate_entry_names_keep_the_last_write() {
    let first = ply::Node::Blob(digest(b"first"));
    let second = ply::Node::Blob(digest(b"second"));

    let tree = ply::Tree::new([("x".to_string(), first), ("x".to_string(), second)], []);
    assert_eq!(tree, ply::Tree::new([("x".to_string(), second)], []));
}

#[test]
fn distinct_names_with_the_same_content_are_both_kept() {
    let content = digest(b"same");
    let two_entries = ply::Tree::new(
        [("a".to_string(), ply::Node::Blob(content)), ("b".to_string(), ply::Node::Blob(content))],
        [],
    );
    let one_entry = ply::Tree::new([("a".to_string(), ply::Node::Blob(content))], []);
    assert_ne!(cas::digest(&two_entries), cas::digest(&one_entry));
}
