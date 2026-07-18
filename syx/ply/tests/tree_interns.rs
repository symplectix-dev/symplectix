//! How `Tree` handles its `interns`.

mod common;
use common::{
    digest,
    digest_bytes,
};

#[test]
fn interns_are_equal_regardless_of_build_order() {
    let a = digest_bytes(b"a");
    let b = digest_bytes(b"b");
    assert_eq!(ply::Tree::new([], [a, b]), ply::Tree::new([], [b, a]));
}

#[test]
fn interning_the_same_digest_twice_does_not_change_the_tree() {
    let a = digest_bytes(b"a");
    assert_eq!(digest(&ply::Tree::new([], [a, a])), digest(&ply::Tree::new([], [a])));
}
