//! `Function`'s digest is deterministic.

mod common;
use common::{
    digest,
    digest_bytes,
};

#[test]
fn hashing_the_same_command_function_twice_gives_the_same_digest() {
    let command = digest_bytes(b"command");
    let a = func::Function::command(command);
    let b = func::Function::command(command);
    assert_eq!(digest(&a), digest(&b));
}

#[test]
fn hashing_the_same_map_function_twice_gives_the_same_digest() {
    let command = digest_bytes(b"command");
    let config = digest_bytes(b"config");
    let a = func::Function::map(command, config);
    let b = func::Function::map(command, config);
    assert_eq!(digest(&a), digest(&b));
}
