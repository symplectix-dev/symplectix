//! Shared fixtures for `ply`'s external test suite.
#![allow(dead_code)]

pub fn digest(bytes: &[u8]) -> cas::Digest {
    let mut h = cas::Hasher::new();
    h.part(bytes);
    h.digest()
}

pub fn command(program: &str, args: &[&str]) -> ply::Command {
    let mut command = ply::Command::new(program);
    command.args(args);
    command
}

pub fn store() -> (testing::TempDir, cas::Store) {
    let dir = testing::tempdir();
    let store = cas::Store::open(dir.path()).unwrap();
    (dir, store)
}
