//! `Function`'s digest is deterministic.

mod common;
use common::digest;

#[test]
fn hashing_the_same_command_function_twice_gives_the_same_digest() {
    let command = digest(b"command");
    let a = ply::Function::command(command);
    let b = ply::Function::command(command);
    assert_eq!(cas::digest(&a), cas::digest(&b));
}

#[test]
fn hashing_the_same_map_function_twice_gives_the_same_digest() {
    let command = digest(b"command");
    let config = digest(b"config");
    let a = ply::Function::map(command, config);
    let b = ply::Function::map(command, config);
    assert_eq!(cas::digest(&a), cas::digest(&b));
}
