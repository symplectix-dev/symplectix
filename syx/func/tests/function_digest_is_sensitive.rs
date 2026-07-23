//! `Function`'s digest is sensitive to its input.

mod common;
use common::{
    digest,
    digest_bytes,
};

#[test]
fn different_command_produces_different_action_function_digests() {
    let config = digest_bytes(b"config");
    let a = func::Function::action(digest_bytes(b"command-a"), config);
    let b = func::Function::action(digest_bytes(b"command-b"), config);
    assert_ne!(digest(&a), digest(&b));
}

#[test]
fn different_config_produces_different_action_function_digests() {
    let command = digest_bytes(b"command");
    let a = func::Function::action(command, digest_bytes(b"config-a"));
    let b = func::Function::action(command, digest_bytes(b"config-b"));
    assert_ne!(digest(&a), digest(&b));
}

#[test]
fn different_command_produces_different_server_function_digests() {
    let config = digest_bytes(b"config");
    let a = func::Function::server(digest_bytes(b"command-a"), config);
    let b = func::Function::server(digest_bytes(b"command-b"), config);
    assert_ne!(digest(&a), digest(&b));
}

#[test]
fn different_config_produces_different_server_function_digests() {
    let command = digest_bytes(b"command");
    let a = func::Function::server(command, digest_bytes(b"config-a"));
    let b = func::Function::server(command, digest_bytes(b"config-b"));
    assert_ne!(digest(&a), digest(&b));
}

#[test]
fn action_and_server_variants_do_not_collide_on_the_same_command_and_config() {
    let command = digest_bytes(b"command");
    let config = digest_bytes(b"config");
    let a = func::Function::action(command, config);
    let b = func::Function::server(command, config);
    assert_ne!(digest(&a), digest(&b));
}
