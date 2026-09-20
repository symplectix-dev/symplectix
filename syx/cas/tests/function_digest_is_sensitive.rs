//! `Function`'s digest is sensitive to its input.

mod common;
use common::Function;

#[test]
fn different_command_produces_different_action_function_digests() {
    let config = cas_testing::digest_bytes(b"config");
    let a = Function::action(cas_testing::digest_bytes(b"command-a"), config);
    let b = Function::action(cas_testing::digest_bytes(b"command-b"), config);
    assert_ne!(cas_testing::digest(&a), cas_testing::digest(&b));
}

#[test]
fn different_config_produces_different_action_function_digests() {
    let command = cas_testing::digest_bytes(b"command");
    let a = Function::action(command, cas_testing::digest_bytes(b"config-a"));
    let b = Function::action(command, cas_testing::digest_bytes(b"config-b"));
    assert_ne!(cas_testing::digest(&a), cas_testing::digest(&b));
}

#[test]
fn different_command_produces_different_server_function_digests() {
    let config = cas_testing::digest_bytes(b"config");
    let a = Function::server(cas_testing::digest_bytes(b"command-a"), config);
    let b = Function::server(cas_testing::digest_bytes(b"command-b"), config);
    assert_ne!(cas_testing::digest(&a), cas_testing::digest(&b));
}

#[test]
fn different_config_produces_different_server_function_digests() {
    let command = cas_testing::digest_bytes(b"command");
    let a = Function::server(command, cas_testing::digest_bytes(b"config-a"));
    let b = Function::server(command, cas_testing::digest_bytes(b"config-b"));
    assert_ne!(cas_testing::digest(&a), cas_testing::digest(&b));
}

#[test]
fn action_and_server_variants_do_not_collide_on_the_same_command_and_config() {
    let command = cas_testing::digest_bytes(b"command");
    let config = cas_testing::digest_bytes(b"config");
    let a = Function::action(command, config);
    let b = Function::server(command, config);
    assert_ne!(cas_testing::digest(&a), cas_testing::digest(&b));
}
