//! Collection: content-addressed file content and collections.
//!
//! `Tree` and `Bag` are internal representations, reached only through
//! `Collection`.

use std::collections::BTreeMap;

use crate::hash::{
    self,
    Digest,
    Hasher,
};
use crate::store::Storable;

/// What a `Tree` entry's name points to: a file's content, or a nested
/// `Tree`, each referenced by digest rather than embedded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Node {
    Blob(Digest),
    Tree(Digest),
}

/// A content-addressed directory: names mapped to a `Blob` (file) or a
/// nested `Tree` (subdirectory). Backed by a `BTreeMap`, so entries are
/// always sorted by name and a name can't appear twice, meaning the same
/// entries built in any order produce the same `Tree`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tree {
    entries: BTreeMap<String, Node>,
}

/// A content-addressed, unordered multiset of blobs: no names, duplicates
/// kept, so the same members built in any order produce the same `Bag`.
/// Use this instead of `Tree` when items have no meaningful name or path
/// -- once they do, use `Tree` instead.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Bag {
    members: Vec<Digest>,
}

impl Tree {
    /// Build a `Tree` from `entries`.
    pub fn new(entries: impl IntoIterator<Item = (String, Node)>) -> Self {
        Tree { entries: entries.into_iter().collect() }
    }

    /// The tree's entries, sorted by name.
    pub fn entries(&self) -> &BTreeMap<String, Node> {
        &self.entries
    }
}

impl Bag {
    /// Build a `Bag` from `members`, sorted.
    pub fn new(members: impl IntoIterator<Item = Digest>) -> Self {
        let mut members: Vec<Digest> = members.into_iter().collect();
        members.sort();
        Bag { members }
    }

    /// The bag's members, sorted (duplicates kept).
    pub fn members(&self) -> &[Digest] {
        &self.members
    }
}

/// A content-addressed collection: either a `Tree` (named, hierarchical)
/// or a `Bag` (unnamed, duplicate-tolerant multiset). Which one it is, and
/// how it's represented internally, is deliberately not exposed -- callers
/// (e.g. `Action`) just build one and read its digest.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Collection {
    blobs:   Blobs,
    /// Additional blobs a producer created while building this
    /// collection but that aren't reachable from any entry's name or bag
    /// member (e.g. images extracted from a PDF that a member only
    /// references opaquely). Recording them here keeps them from looking
    /// unreferenced to a GC walking the CAS from live roots. This applies
    /// equally regardless of whether the collection is a `Tree` or a
    /// `Bag`, so it lives here rather than being duplicated on each.
    interns: Vec<Digest>,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Blobs {
    Tree(Tree),
    Bag(Bag),
}

impl Collection {
    /// A `Collection` backed by a `Tree`.
    pub fn tree(
        entries: impl IntoIterator<Item = (String, Node)>,
        interns: impl IntoIterator<Item = Digest>,
    ) -> Self {
        let mut interns: Vec<Digest> = interns.into_iter().collect();
        interns.sort();
        Collection { blobs: Blobs::Tree(Tree::new(entries)), interns }
    }

    /// A `Collection` backed by a `Bag`.
    pub fn bag(
        members: impl IntoIterator<Item = Digest>,
        interns: impl IntoIterator<Item = Digest>,
    ) -> Self {
        let mut interns: Vec<Digest> = interns.into_iter().collect();
        interns.sort();
        Collection { blobs: Blobs::Bag(Bag::new(members)), interns }
    }
}

impl Storable for Collection {}

impl TryFrom<&[u8]> for Collection {
    type Error = cbor2::de::Error;

    /// Deserialize `bytes` (a CBOR encoding, canonical or not) as a
    /// `Collection`.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        cbor2::from_slice(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(bytes: &[u8]) -> Digest {
        let mut h = Hasher::new();
        h.part(bytes);
        h.digest()
    }

    #[test]
    fn empty_tree_entries_is_empty() {
        assert!(Tree::new([]).entries().is_empty());
    }

    #[test]
    fn tree_entries_are_sorted_by_name() {
        let a = ("a".to_string(), Node::Blob(digest(b"a")));
        let b = ("b".to_string(), Node::Blob(digest(b"b")));

        let forward = Tree::new([a.clone(), b.clone()]);
        let backward = Tree::new([b, a]);

        assert_eq!(forward.entries(), backward.entries());
    }

    #[test]
    fn duplicate_name_keeps_last_write() {
        let first = Node::Blob(digest(b"first"));
        let second = Node::Blob(digest(b"second"));

        let tree = Tree::new([("x".to_string(), first), ("x".to_string(), second)]);

        assert_eq!(tree.entries().len(), 1);
        assert_eq!(tree.entries().get("x"), Some(&second));
    }

    #[test]
    fn bag_keeps_duplicate_members() {
        let a = digest(b"a");
        let bag = Bag::new([a, a]);
        assert_eq!(bag.members(), [a, a]);
    }

    #[test]
    fn bag_members_are_sorted() {
        let a = digest(b"a");
        let b = digest(b"b");
        assert_eq!(Bag::new([a, b]).members(), Bag::new([b, a]).members());
    }

    #[test]
    fn empty_tree_digest_is_deterministic() {
        assert_eq!(
            hash::digest_of(&Collection::tree([], [])),
            hash::digest_of(&Collection::tree([], []))
        );
    }

    #[test]
    fn tree_digest_ignores_build_order() {
        let a = ("a".to_string(), Node::Blob(digest(b"a")));
        let b = ("b".to_string(), Node::Blob(digest(b"b")));

        let forward = Collection::tree([a.clone(), b.clone()], []);
        let backward = Collection::tree([b, a], []);

        assert_eq!(hash::digest_of(&forward), hash::digest_of(&backward));
    }

    #[test]
    fn tree_digest_depends_on_entry_names() {
        let blob = digest(b"content");
        let a = Collection::tree([("a".to_string(), Node::Blob(blob))], []);
        let b = Collection::tree([("b".to_string(), Node::Blob(blob))], []);
        assert_ne!(hash::digest_of(&a), hash::digest_of(&b));
    }

    #[test]
    fn tree_digest_distinguishes_blob_from_tree() {
        // A blob and a nested tree that happen to wrap the same inner
        // digest must not collide.
        let inner = digest(b"same");
        let a = Collection::tree([("x".to_string(), Node::Blob(inner))], []);
        let b = Collection::tree([("x".to_string(), Node::Tree(inner))], []);
        assert_ne!(hash::digest_of(&a), hash::digest_of(&b));
    }

    #[test]
    fn tree_digest_depends_on_nested_tree_content() {
        let inner_a = Collection::tree([("f".to_string(), Node::Blob(digest(b"a")))], []);
        let inner_b = Collection::tree([("f".to_string(), Node::Blob(digest(b"b")))], []);

        let a =
            Collection::tree([("dir".to_string(), Node::Tree(hash::digest_of(&inner_a)))], []);
        let b =
            Collection::tree([("dir".to_string(), Node::Tree(hash::digest_of(&inner_b)))], []);
        assert_ne!(hash::digest_of(&a), hash::digest_of(&b));
    }

    #[test]
    fn tree_digest_depends_on_interns() {
        let a = Collection::tree([], [digest(b"intern-a")]);
        let b = Collection::tree([], [digest(b"intern-b")]);
        assert_ne!(hash::digest_of(&a), hash::digest_of(&b));
    }

    #[test]
    fn tree_interns_ignore_build_order() {
        let a = digest(b"a");
        let b = digest(b"b");
        assert_eq!(
            hash::digest_of(&Collection::tree([], [a, b])),
            hash::digest_of(&Collection::tree([], [b, a]))
        );
    }

    #[test]
    fn empty_bag_digest_is_deterministic() {
        assert_eq!(
            hash::digest_of(&Collection::bag([], [])),
            hash::digest_of(&Collection::bag([], []))
        );
    }

    #[test]
    fn bag_digest_ignores_build_order() {
        let a = digest(b"a");
        let b = digest(b"b");
        assert_eq!(
            hash::digest_of(&Collection::bag([a, b], [])),
            hash::digest_of(&Collection::bag([b, a], []))
        );
    }

    #[test]
    fn bag_digest_depends_on_interns() {
        let a = Collection::bag([], [digest(b"intern-a")]);
        let b = Collection::bag([], [digest(b"intern-b")]);
        assert_ne!(hash::digest_of(&a), hash::digest_of(&b));
    }

    #[test]
    fn bag_digest_depends_on_member_count() {
        // Two members vs. one member "shared" as both a member and an
        // intern must not collide: the member-count prefix disambiguates.
        let a = digest(b"a");
        let b = digest(b"b");
        let two_members = Collection::bag([a, b], []);
        let one_member_one_intern = Collection::bag([a], [b]);
        assert_ne!(hash::digest_of(&two_members), hash::digest_of(&one_member_one_intern));
    }

    #[test]
    fn empty_tree_and_empty_bag_with_same_interns_do_not_collide() {
        // Same shape once framed (a count of 0, then the same interns) --
        // only the kind tag keeps these apart.
        let interns = [digest(b"x"), digest(b"y")];
        let tree = Collection::tree([], interns);
        let bag = Collection::bag([], interns);
        assert_ne!(hash::digest_of(&tree), hash::digest_of(&bag));
    }

    #[test]
    fn collection_digest_matches_hash_of_its_own_canonical_cbor() {
        // The invariant `Store` relies on: a `Collection`'s digest equals
        // the digest of its own canonical CBOR bytes, same as every other
        // content-addressed type here.
        let entries = [("a".to_string(), Node::Blob(digest(b"a")))];
        let interns = [digest(b"intern")];
        let collection = Collection::tree(entries, interns);

        let bytes = cbor2::to_canonical_vec(&collection).unwrap();
        let mut h = Hasher::new();
        h.part(bytes);
        assert_eq!(hash::digest_of(&collection), h.digest());
    }

    #[test]
    fn collection_tree_and_bag_do_not_collide() {
        let interns = [digest(b"x"), digest(b"y")];
        let tree = Collection::tree([], interns);
        let bag = Collection::bag([], interns);
        assert_ne!(hash::digest_of(&tree), hash::digest_of(&bag));
    }

    #[test]
    fn node_round_trips_through_cbor() {
        let want = Node::Blob(digest(b"a"));
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got: Node = cbor2::from_slice(&bytes).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn collection_tree_try_from_bytes_round_trips() {
        let want = Collection::tree([("a".to_string(), Node::Blob(digest(b"a")))], []);
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got = Collection::try_from(bytes.as_slice()).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn collection_bag_try_from_bytes_round_trips() {
        let want = Collection::bag([digest(b"a"), digest(b"b")], []);
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got = Collection::try_from(bytes.as_slice()).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn collection_try_from_rejects_garbage_bytes() {
        assert!(Collection::try_from(&b"not cbor"[..]).is_err());
    }
}
