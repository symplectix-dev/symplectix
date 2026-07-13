//! Action: a content-addressed reference to something runnable.

use std::collections::BTreeMap;

use crate::hash::{
    self,
    Digest,
    Hasher,
};

/// A content-addressed reference to something runnable: a `Command` and
/// the input it runs against, each referenced by digest rather than
/// embedded. Identical command + input always produce the same `Action`
/// digest, so identical runs dedup, and any change to either changes it,
/// so a recorded digest is tamper-evident.
///
/// Scheduling concerns (which worker, which OS/container image to run in)
/// are deliberately not part of this: they're metadata an orchestrator
/// carries alongside an Action's digest when dispatching it, not part of
/// what gets cached/addressed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Action {
    /// Digest of the `Command` to run.
    pub command: Digest,
    /// Digest of the input (a `Tree` or `Bag`, via `Collection`) to run
    /// the command against.
    pub input: Digest,
}

impl Action {
    /// Digest of the action itself, combining `command` and `input` in
    /// order.
    pub fn digest(&self) -> Digest {
        hash::digest_of(self)
    }
}

impl From<Action> for Digest {
    fn from(action: Action) -> Self {
        action.digest()
    }
}

impl TryFrom<&[u8]> for Action {
    type Error = cbor2::de::Error;

    /// Deserialize `bytes` (a CBOR encoding, canonical or not) as an
    /// `Action`.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        cbor2::from_slice(bytes)
    }
}

/// What an `Action` runs: an argv and the environment variables to invoke
/// it with.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Command {
    pub arguments: Vec<String>,
    pub env:       BTreeMap<String, String>,
}

impl Command {
    /// Digest of the command's content.
    pub fn digest(&self) -> Digest {
        hash::digest_of(self)
    }
}

impl From<Command> for Digest {
    fn from(command: Command) -> Self {
        command.digest()
    }
}

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

    fn command(args: &[&str]) -> Command {
        Command {
            arguments: args.iter().map(|s| s.to_string()).collect(),
            env:       BTreeMap::new(),
        }
    }

    #[test]
    fn command_digest_is_deterministic() {
        let a = command(&["echo", "hi"]);
        let b = command(&["echo", "hi"]);
        assert_eq!(a.digest(), b.digest());
    }

    #[test]
    fn command_digest_depends_on_arguments() {
        let a = command(&["echo", "a"]);
        let b = command(&["echo", "b"]);
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn command_digest_depends_on_env() {
        let mut a = command(&["run"]);
        a.env.insert("KEY".to_string(), "a".to_string());
        let mut b = command(&["run"]);
        b.env.insert("KEY".to_string(), "b".to_string());
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn command_round_trips_through_cbor() {
        let want = command(&["echo", "hi"]);
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got: Command = cbor2::from_slice(&bytes).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn command_try_from_bytes_round_trips() {
        let want = command(&["echo", "hi"]);
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
        let a = Action { command, input };
        let b = Action { command, input };
        assert_eq!(a.digest(), b.digest());
    }

    #[test]
    fn action_digest_changes_with_command() {
        let input = digest(b"input");
        let a = Action { command: digest(b"command-a"), input };
        let b = Action { command: digest(b"command-b"), input };
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn action_digest_changes_with_input() {
        let command = digest(b"command");
        let a = Action { command, input: digest(b"input-a") };
        let b = Action { command, input: digest(b"input-b") };
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn action_round_trips_through_cbor() {
        let want = Action { command: digest(b"command"), input: digest(b"input") };
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got: Action = cbor2::from_slice(&bytes).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn action_try_from_bytes_round_trips() {
        let want = Action { command: digest(b"command"), input: digest(b"input") };
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got = Action::try_from(bytes.as_slice()).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn action_try_from_rejects_garbage_bytes() {
        assert!(Action::try_from(&b"not cbor"[..]).is_err());
    }
}
