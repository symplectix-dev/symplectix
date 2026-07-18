//! How `Tree` handles its `interns`.

mod common;
use common::digest;

#[test]
fn interns_are_equal_regardless_of_build_order() {
    let a = digest(b"a");
    let b = digest(b"b");
    assert_eq!(ply::Tree::new([], [a, b]), ply::Tree::new([], [b, a]));
}

#[test]
fn interning_the_same_digest_twice_does_not_change_the_tree() {
    let a = digest(b"a");
    assert_eq!(cas::digest(&ply::Tree::new([], [a, a])), cas::digest(&ply::Tree::new([], [a])));
}
