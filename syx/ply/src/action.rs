//! Action: a content-addressed reference to something runnable.

use std::collections::BTreeMap;

use crate::hash::{
    self,
    Digest,
    Hasher,
};
use crate::store::Storable;

/// A content-addressed reference to something runnable: a `Command` and
/// the input it runs against, each referenced by digest rather than
/// embedded. Identical command + input always produce the same `Action`
/// digest (via `hash::digest_of`), so identical runs dedup, and any
/// change to either changes it, so a recorded digest is tamper-evident.
///
/// Scheduling concerns (which worker, which OS/container image to run in)
/// are deliberately not part of this: they're metadata an orchestrator
/// carries alongside an Action's digest when dispatching it, not part of
/// what gets cached/addressed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Action {
    command: Digest,
    input:   Digest,
}

impl Action {
    /// An `Action` running `command` against `input`.
    pub fn new(command: Digest, input: Digest) -> Self {
        Action { command, input }
    }
}

impl Storable for Action {}

impl TryFrom<&[u8]> for Action {
    type Error = cbor2::de::Error;

    /// Deserialize `bytes` (a CBOR encoding, canonical or not) as an
    /// `Action`.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        cbor2::from_slice(bytes)
    }
}

/// What an `Action` runs: a program, its arguments, and the environment
/// variables to invoke it with. Named to match `std::process::Command`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

    pub fn arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for arg in args {
            self.arg(arg);
        }
        self
    }

    pub fn env<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.env.insert(key.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (key, value) in vars {
            self.env(key, value);
        }
        self
    }
}

impl Storable for Command {}

impl TryFrom<&[u8]> for Command {
    type Error = cbor2::de::Error;

    /// Deserialize `bytes` (a CBOR encoding, canonical or not) as a
    /// `Command`.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        cbor2::from_slice(bytes)
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

    fn command(program: &str, args: &[&str]) -> Command {
        let mut command = Command::new(program);
        command.args(args);
        command
    }

    #[test]
    fn command_digest_is_deterministic() {
        let a = command("echo", &["hi"]);
        let b = command("echo", &["hi"]);
        assert_eq!(hash::digest_of(&a), hash::digest_of(&b));
    }

    #[test]
    fn command_digest_depends_on_program() {
        let a = command("echo", &["hi"]);
        let b = command("cat", &["hi"]);
        assert_ne!(hash::digest_of(&a), hash::digest_of(&b));
    }

    #[test]
    fn command_digest_depends_on_args() {
        let a = command("echo", &["a"]);
        let b = command("echo", &["b"]);
        assert_ne!(hash::digest_of(&a), hash::digest_of(&b));
    }

    #[test]
    fn command_digest_depends_on_env() {
        let mut a = Command::new("run");
        a.env("KEY", "a");
        let mut b = Command::new("run");
        b.env("KEY", "b");
        assert_ne!(hash::digest_of(&a), hash::digest_of(&b));
    }

    #[test]
    fn command_round_trips_through_cbor() {
        let want = command("echo", &["hi"]);
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got: Command = cbor2::from_slice(&bytes).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn command_try_from_bytes_round_trips() {
        let want = command("echo", &["hi"]);
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got = Command::try_from(bytes.as_slice()).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn command_try_from_rejects_garbage_bytes() {
        assert!(Command::try_from(&b"not cbor"[..]).is_err());
    }

    #[test]
    fn action_digest_is_deterministic() {
        let command = digest(b"command");
        let input = digest(b"input");
        let a = Action::new(command, input);
        let b = Action::new(command, input);
        assert_eq!(hash::digest_of(&a), hash::digest_of(&b));
    }

    #[test]
    fn action_digest_changes_with_command() {
        let input = digest(b"input");
        let a = Action::new(digest(b"command-a"), input);
        let b = Action::new(digest(b"command-b"), input);
        assert_ne!(hash::digest_of(&a), hash::digest_of(&b));
    }

    #[test]
    fn action_digest_changes_with_input() {
        let command = digest(b"command");
        let a = Action::new(command, digest(b"input-a"));
        let b = Action::new(command, digest(b"input-b"));
        assert_ne!(hash::digest_of(&a), hash::digest_of(&b));
    }

    #[test]
    fn action_round_trips_through_cbor() {
        let want = Action::new(digest(b"command"), digest(b"input"));
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got: Action = cbor2::from_slice(&bytes).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn action_try_from_bytes_round_trips() {
        let want = Action::new(digest(b"command"), digest(b"input"));
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got = Action::try_from(bytes.as_slice()).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn action_try_from_rejects_garbage_bytes() {
        assert!(Action::try_from(&b"not cbor"[..]).is_err());
    }

    #[tokio::test]
    async fn action_and_its_command_and_input_resolve_from_store() {
        use crate::blob::{
            Collection,
            Node,
        };
        use crate::store::Store;

        let dir = testing::tempdir();
        let store = Store::new(dir.path(), 100);

        // The input: a Tree with one file entry, itself resolvable from
        // the store.
        let file_digest = store.put(&b"print('hi')".to_vec()).await.unwrap();
        let input = Collection::tree([("main.py".to_string(), Node::Blob(file_digest))], []);
        let input_digest = store.put(&input).await.unwrap();

        // The command to run against that input.
        let mut command = Command::new("python3");
        command.arg("main.py");
        let command_digest = store.put(&command).await.unwrap();

        // The action tying command and input together.
        let action = Action::new(command_digest, input_digest);
        let action_digest = store.put(&action).await.unwrap();

        // Read the whole graph back out of the store using only the
        // action's digest -- the resolution an executor would do before
        // actually running it. Reaches into `command`/`input` directly
        // (private fields, but this test lives inside the same module).
        let resolved_action: Action = store.get(&action_digest).await.unwrap().unwrap();
        assert_eq!(resolved_action, action);

        let resolved_command: Command = store.get(&resolved_action.command).await.unwrap().unwrap();
        assert_eq!(resolved_command, command);

        let resolved_input: Collection = store.get(&resolved_action.input).await.unwrap().unwrap();
        assert_eq!(resolved_input, input);

        // The file the input tree references is itself resolvable.
        assert_eq!(store.get(&file_digest).await.unwrap(), Some(b"print('hi')".to_vec()));
    }
}
