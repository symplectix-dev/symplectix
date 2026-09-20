//! How `Tree` handles its `entries`.

mod common;
use common::{
    Node,
    Tree,
};

#[test]
fn duplicate_entry_names_keep_the_last_write() {
    let first = Node::Blob(cas_testing::digest_bytes(b"first"));
    let second = Node::Blob(cas_testing::digest_bytes(b"second"));

    let tree = Tree::new([("x".to_string(), first), ("x".to_string(), second)], []);
    assert_eq!(tree, Tree::new([("x".to_string(), second)], []));
}

#[test]
fn distinct_names_with_the_same_content_are_both_kept() {
    let content = cas_testing::digest_bytes(b"same");
    let two_entries = Tree::new(
        [("a".to_string(), Node::Blob(content)), ("b".to_string(), Node::Blob(content))],
        [],
    );
    let one_entry = Tree::new([("a".to_string(), Node::Blob(content))], []);
    assert_ne!(cas_testing::digest(&two_entries), cas_testing::digest(&one_entry));
}
