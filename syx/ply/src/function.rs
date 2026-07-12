//! Function: a content-addressed reference to something runnable.

use crate::hash::{
    Digest,
    Hasher,
};

/// The configuration to invoke a `Function`'s runner with (entrypoint, env,
/// resources, ...).
///
/// Shape still evolving -- currently just enough to be
/// content-addressable.
pub struct Config {
    pub name: String,
}

impl Config {
    /// Digest of the config's content.
    pub fn digest(&self) -> Digest {
        let mut h = Hasher::new();
        h.part(self.name.as_bytes());
        h.digest()
    }
}

/// A content-addressed reference to something runnable: a runner (e.g. an
/// OCI image) and the config to invoke it with, each referenced by digest
/// rather than embedded. Identical runner + config always produce the same
/// `Function` digest, so identical runs dedup, and any change to either
/// input changes it, so a recorded digest is tamper-evident.
pub struct Function {
    /// Digest of the runner to run (e.g. an OCI image's manifest digest).
    pub runner: Digest,
    /// Digest of the `Config` to run the runner with.
    pub config: Digest,
}

impl Function {
    /// Digest of the function itself, combining `runner` and `config` in
    /// order.
    pub fn digest(&self) -> Digest {
        let mut h = Hasher::new();
        h.part(self.runner.as_bytes());
        h.part(self.config.as_bytes());
        h.digest()
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
    fn config_digest_is_deterministic() {
        let a = Config { name: "foo".to_string() };
        let b = Config { name: "foo".to_string() };
        assert_eq!(a.digest(), b.digest());
    }

    #[test]
    fn config_digest_depends_on_name() {
        let a = Config { name: "a".to_string() };
        let b = Config { name: "b".to_string() };
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn function_digest_is_deterministic() {
        let runner = digest(b"runner");
        let config = digest(b"config");
        let a = Function { runner: runner.clone(), config: config.clone() };
        let b = Function { runner, config };
        assert_eq!(a.digest(), b.digest());
    }

    #[test]
    fn function_digest_changes_with_runner() {
        let config = digest(b"config");
        let a = Function { runner: digest(b"runner-a"), config: config.clone() };
        let b = Function { runner: digest(b"runner-b"), config };
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn function_digest_changes_with_config() {
        let runner = digest(b"runner");
        let a = Function { runner: runner.clone(), config: digest(b"config-a") };
        let b = Function { runner, config: digest(b"config-b") };
        assert_ne!(a.digest(), b.digest());
    }
}
