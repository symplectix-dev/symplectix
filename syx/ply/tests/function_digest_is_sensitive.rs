//! `Function`'s digest is sensitive to its input.

mod common;
use common::{
    digest,
    digest_bytes,
};

#[test]
fn different_command_produces_different_command_function_digests() {
    let a = ply::Function::command(digest_bytes(b"command-a"));
    let b = ply::Function::command(digest_bytes(b"command-b"));
    assert_ne!(digest(&a), digest(&b));
}

#[test]
fn different_command_produces_different_map_function_digests() {
    let config = digest_bytes(b"config");
    let a = ply::Function::map(digest_bytes(b"command-a"), config);
    let b = ply::Function::map(digest_bytes(b"command-b"), config);
    assert_ne!(digest(&a), digest(&b));
}

#[test]
fn different_config_produces_different_map_function_digests() {
    let command = digest_bytes(b"command");
    let a = ply::Function::map(command, digest_bytes(b"config-a"));
    let b = ply::Function::map(command, digest_bytes(b"config-b"));
    assert_ne!(digest(&a), digest(&b));
}

#[test]
fn map_and_reduce_with_the_same_command_and_config_have_different_digests() {
    let command = digest_bytes(b"command");
    let config = digest_bytes(b"config");
    let map = ply::Function::map(command, config);
    let reduce = ply::Function::reduce(command, config);
    assert_ne!(digest(&map), digest(&reduce));
}

#[test]
fn command_and_map_variants_do_not_collide_on_the_same_command() {
    let command = digest_bytes(b"command");
    let a = ply::Function::command(command);
    let b = ply::Function::map(command, digest_bytes(b"config"));
    assert_ne!(digest(&a), digest(&b));
}
