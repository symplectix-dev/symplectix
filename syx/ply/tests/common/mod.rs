//! Shared fixtures for `ply`'s external test suite.
#![allow(dead_code)]

pub fn digest_bytes(bytes: &[u8]) -> cas::Digest {
    let mut h = cas::Hasher::new();
    h.part(bytes);
    h.digest()
}

pub fn command(program: &str, args: &[&str]) -> ply::Command {
    let mut command = ply::Command::new(program);
    command.args(args);
    command
}

pub fn store() -> (testing::TempDir, ply::Store) {
    let dir = testing::tempdir();
    let store = ply::Store::open(dir.path(), 16 * 1024 * 1024).unwrap();
    (dir, store)
}

/// `cas::digest`, unwrapped: every `ToBytes` impl used in this suite is
/// expected to succeed, so tests don't need to handle the error case.
pub fn digest<T: cas::ToBytes>(value: &T) -> cas::Digest {
    cas::digest(value).unwrap()
}

/// `len` random bytes, which zstd can't meaningfully shrink.
pub fn incompressible_bytes(len: usize) -> Vec<u8> {
    use rand::RngExt as _;

    let mut out = vec![0u8; len];
    rand::rng().fill(&mut out[..]);
    out
}
