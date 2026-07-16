//! Tree: a content-addressed collection of blobs.

use std::collections::BTreeMap;

/// What a `Tree` entry's name points to: a file's content, or a nested
/// `Tree`, each referenced by digest rather than embedded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Node {
    Blob(cas::Digest),
    Tree(cas::Digest),
}

/// A content-addressed tree of blobs: names mapped to a `Blob` or a
/// nested `Tree`. Entries are sorted by name and a name can't appear
/// twice, meaning the same entries built in any order produce the same
/// `Tree`. Two entries may still point at the same underlying digest
/// (e.g. two identically-named-differently files with the same content).
/// Uniqueness is on the name, not the value, so `Tree` can represent a
/// multiset of items as long as each has a distinct name, which is the
/// normal case (files always have distinct paths).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tree {
    entries: BTreeMap<String, Node>,
    /// Additional blobs a producer created while building this tree but
    /// that aren't reachable from any entry's name.
    /// Recording them here keeps them from looking unreferenced to a GC
    /// walking the CAS from live roots -- `entries` alone can't express
    /// "reachable but not a real file", since every entry is materialized.
    interns: Vec<cas::Digest>,
}

impl Tree {
    /// Build a `Tree` from `entries` and `interns`.
    pub fn new(
        entries: impl IntoIterator<Item = (String, Node)>,
        interns: impl IntoIterator<Item = cas::Digest>,
    ) -> Self {
        let mut interns: Vec<cas::Digest> = interns.into_iter().collect();
        interns.sort();
        Tree { entries: entries.into_iter().collect(), interns }
    }
}

impl cas::Storable for Tree {}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(bytes: &[u8]) -> cas::Digest {
        let mut h = cas::Hasher::new();
        h.part(bytes);
        h.digest()
    }

    #[test]
    fn empty_tree_digest_is_deterministic() {
        assert_eq!(cas::digest(&Tree::new([], [])), cas::digest(&Tree::new([], [])));
    }

    #[test]
    fn tree_digest_ignores_build_order() {
        let a = ("a".to_string(), Node::Blob(digest(b"a")));
        let b = ("b".to_string(), Node::Blob(digest(b"b")));

        let forward = Tree::new([a.clone(), b.clone()], []);
        let backward = Tree::new([b, a], []);

        assert_eq!(cas::digest(&forward), cas::digest(&backward));
        assert_eq!(forward.entries, backward.entries);
    }

    #[test]
    fn duplicate_name_keeps_last_write() {
        let first = Node::Blob(digest(b"first"));
        let second = Node::Blob(digest(b"second"));

        let tree = Tree::new([("x".to_string(), first), ("x".to_string(), second)], []);

        assert_eq!(tree.entries.len(), 1);
        assert_eq!(tree.entries.get("x"), Some(&second));
    }

    #[test]
    fn tree_keeps_duplicate_content_under_distinct_names() {
        let content = digest(b"same");
        let tree = Tree::new(
            [("a".to_string(), Node::Blob(content)), ("b".to_string(), Node::Blob(content))],
            [],
        );
        assert_eq!(tree.entries.len(), 2);
    }

    #[test]
    fn tree_digest_depends_on_entry_names() {
        let blob = digest(b"content");
        let a = Tree::new([("a".to_string(), Node::Blob(blob))], []);
        let b = Tree::new([("b".to_string(), Node::Blob(blob))], []);
        assert_ne!(cas::digest(&a), cas::digest(&b));
    }

    #[test]
    fn tree_digest_distinguishes_blob_from_tree() {
        // A blob and a nested tree that happen to wrap the same inner
        // digest must not collide.
        let inner = digest(b"same");
        let a = Tree::new([("x".to_string(), Node::Blob(inner))], []);
        let b = Tree::new([("x".to_string(), Node::Tree(inner))], []);
        assert_ne!(cas::digest(&a), cas::digest(&b));
    }

    #[test]
    fn tree_digest_depends_on_nested_tree_content() {
        let inner_a = Tree::new([("f".to_string(), Node::Blob(digest(b"a")))], []);
        let inner_b = Tree::new([("f".to_string(), Node::Blob(digest(b"b")))], []);

        let a = Tree::new([("dir".to_string(), Node::Tree(cas::digest(&inner_a)))], []);
        let b = Tree::new([("dir".to_string(), Node::Tree(cas::digest(&inner_b)))], []);
        assert_ne!(cas::digest(&a), cas::digest(&b));
    }

    #[test]
    fn tree_digest_depends_on_interns() {
        let a = Tree::new([], [digest(b"intern-a")]);
        let b = Tree::new([], [digest(b"intern-b")]);
        assert_ne!(cas::digest(&a), cas::digest(&b));
    }

    #[test]
    fn tree_interns_ignore_build_order() {
        let a = digest(b"a");
        let b = digest(b"b");
        assert_eq!(cas::digest(&Tree::new([], [a, b])), cas::digest(&Tree::new([], [b, a])));
    }

    #[test]
    fn tree_interns_are_sorted() {
        let a = digest(b"a");
        let b = digest(b"b");
        assert_eq!(Tree::new([], [a, b]).interns, Tree::new([], [b, a]).interns);
    }
}
