//! Shared fixtures for `forgetter`'s external test suite.
//!
//! `Command`/`Function`/`Tree`/`Node` are test-only illustrations of what
//! gets content-addressed on top of `forgetter`, not part of its public
//! API.
#![allow(dead_code)]

use std::collections::{
    BTreeMap,
    BTreeSet,
};
use std::path::Path;
use std::sync::Arc;

use content_addressing as cas;
use object_store::ObjectStore;

/// A program, its arguments, and the environment variables to invoke it
/// with. Shared by:
/// - `Function::Action` (run once)
/// - `Function::Server` (kept warm across calls, invoked repeatedly)
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    cas_cbor2::ToBytes,
    cas_cbor2::FromBytes,
)]
pub struct Command {
    program: String,
    args:    Vec<String>,
    env:     BTreeMap<String, String>,
}

impl Command {
    /// A `Command` running `program` with `args` and `env`.
    pub fn new(program: impl Into<String>) -> Self {
        Command { program: program.into(), args: Vec::new(), env: BTreeMap::new() }
    }

    /// Append one argument.
    pub fn arg<S>(mut self, arg: S) -> Self
    where
        S: AsRef<str>,
    {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    /// Append each argument, in order.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.args.extend(args.into_iter().map(|a| a.as_ref().to_owned()));
        self
    }

    /// Set one environment variable.
    pub fn env<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.env.insert(key.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    /// Set each environment variable, in order.
    pub fn envs<I, K, V>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.env
            .extend(vars.into_iter().map(|(k, v)| (k.as_ref().to_owned(), v.as_ref().to_owned())));
        self
    }
}

/// A reference to something runnable, addressed by how it's invoked:
///
/// - `Action`: run once, directly, against input supplied at call time. Cached at the granularity
///   of the whole input: "has this exact input been processed before?".
/// - `Server`: independent, per-blob calls to a server process. Cached per blob, not per call: "has
///   this specific item been processed before?".
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    cas_cbor2::ToBytes,
    cas_cbor2::FromBytes,
)]
pub enum Function {
    /// Run once, directly.
    Action {
        /// The program to run.
        command: cas::Digest,
        /// Its configuration, a `Tree`, materialized before `command` runs.
        config:  cas::Digest,
    },
    /// Call a process kept warm across independent, per-blob requests.
    Server {
        /// The process to call, started on demand and shut down when idle.
        command: cas::Digest,
        /// Its configuration, a `Tree`.
        config:  cas::Digest,
    },
}

impl Function {
    /// Run `command` once, directly, configured by `config` (a `Tree`),
    /// against input supplied at call time.
    pub fn action(command: cas::Digest, config: cas::Digest) -> Self {
        Function::Action { command, config }
    }

    /// Run `server`, kept warm across calls, configured by `config` (a
    /// `Tree`), called with independent per-item requests.
    pub fn server(command: cas::Digest, config: cas::Digest) -> Self {
        Function::Server { command, config }
    }
}

/// What a `Tree` entry's name points to: a file's content, or a nested
/// `Tree`, each referenced by digest rather than embedded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Node {
    /// A file's content.
    Blob(cas::Digest),
    /// A nested `Tree`.
    Tree(cas::Digest),
}

/// A content-addressed tree of blobs: names mapped to a `Blob` or a
/// nested `Tree`. Entries are sorted by name and a name can't appear
/// twice, meaning the same entries built in any order produce the same
/// `Tree`. Two entries may still point at the same underlying digest,
/// for example two differently named files with the same content.
/// Uniqueness is on the name, not the value, so `Tree` can represent a
/// multiset of items as long as each has a distinct name, which is the
/// normal case, since files always have distinct paths.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    cas_cbor2::ToBytes,
    cas_cbor2::FromBytes,
)]
pub struct Tree {
    entries: BTreeMap<String, Node>,
    /// Additional blobs a producer created while building this tree but
    /// that aren't reachable from any entry's name.
    /// Recording them here keeps them from looking unreferenced to a GC
    /// walking the CAS from live roots, since `entries` alone can't
    /// express "reachable but not a real file" when every entry is
    /// materialized. A digest is either reachable via this tree or not,
    /// so this is a set: interning the same digest twice doesn't change
    /// the tree.
    interns: BTreeSet<cas::Digest>,
}

impl Tree {
    /// Build a `Tree` from `entries` and `interns`.
    pub fn new(
        entries: impl IntoIterator<Item = (String, Node)>,
        interns: impl IntoIterator<Item = cas::Digest>,
    ) -> Self {
        Tree { entries: entries.into_iter().collect(), interns: interns.into_iter().collect() }
    }
}

pub fn command(program: &str, args: &[&str]) -> Command {
    Command::new(program).args(args)
}

/// A `Forgetter` backed by a local-filesystem `ObjectStore` rooted at
/// `root`, staging not-yet-packed blobs in a `logger` subdirectory of
/// `root`.
pub async fn forgetter(root: impl AsRef<Path>) -> forgetter::Forgetter {
    let root = root.as_ref();
    let backend: Arc<dyn ObjectStore> =
        Arc::new(object_store::local::LocalFileSystem::new_with_prefix(root).unwrap());
    forgetter::Forgetter::builder(root.join("logger"))
        .db_prefix("test")
        .db_backend(backend)
        .build()
        .await
        .unwrap()
}

/// A `Forgetter` backed by a local temporary directory.
pub async fn temp_forgetter() -> (testing::TempDir, forgetter::Forgetter) {
    let dir = testing::tempdir();
    let f = forgetter(dir.path()).await;
    (dir, f)
}
