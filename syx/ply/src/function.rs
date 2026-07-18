//! Function: a content-addressed reference to something runnable, in one
//! of three calling conventions.

use std::collections::BTreeMap;

/// A program, its arguments, and the environment variables to invoke it
/// with. Named to match `std::process::Command`. Shared by
/// `Function::Command` (run once) and `Function::Map`/`Function::Reduce`
/// (kept running, invoked repeatedly).
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

/// A reference to something runnable, addressed by how it's invoked:
///
/// - `Command`: run once, directly, against input supplied at call time -- not part of this digest,
///   since it's a specific run's data, not this reference's own identity. Combined dynamically
///   (e.g. for cache lookups) by whatever executes it.
/// - `Map`: independent, per-item calls to an already-running, persistent process (embarrassingly
///   parallel).
/// - `Reduce`: a sequential accumulate-then-finalize call to an already-running, persistent
///   process.
///
/// TODO: add a field for which OCI image the VM boots from. The image is
/// used directly as the VM's boot rootfs; `config` is then materialized inside
/// the booted VM, on top of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Function {
    Command(cas::Digest),
    Map { command: cas::Digest, config: cas::Digest },
    Reduce { command: cas::Digest, config: cas::Digest },
}

impl Function {
    /// Run `command` once, directly, against input supplied at call time.
    pub fn command(command: cas::Digest) -> Self {
        Function::Command(command)
    }

    /// Run `command` as a persistent process, configured by `config` (a
    /// `Tree`), called with independent per-item requests.
    pub fn map(command: cas::Digest, config: cas::Digest) -> Self {
        Function::Map { command, config }
    }

    /// Run `command` as a persistent process, configured by `config` (a
    /// `Tree`), called with a sequential accumulate-then-finalize
    /// request.
    pub fn reduce(command: cas::Digest, config: cas::Digest) -> Self {
        Function::Reduce { command, config }
    }
}
