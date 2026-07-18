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

cas::storable!(Command);

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
/// used directly as the VM's boot rootfs; `config`'s `Tree` is then
/// materialized inside the already-booted VM, on top of it. The two are
/// not merged or resolved into each other here.
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

cas::storable!(Function);

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(bytes: &[u8]) -> cas::Digest {
        let mut h = cas::Hasher::new();
        h.part(bytes);
        h.digest()
    }

    fn command(program: &str, args: &[&str]) -> Command {
        let mut command = Command::new(program);
        command.args(args);
        command
    }

    fn store() -> (testing::TempDir, cas::Store) {
        let dir = testing::tempdir();
        let store = cas::Store::create(dir.path()).unwrap();
        (dir, store)
    }

    #[test]
    fn command_struct_digest_is_deterministic() {
        let a = command("echo", &["hi"]);
        let b = command("echo", &["hi"]);
        assert_eq!(cas::digest(&a), cas::digest(&b));
    }

    #[test]
    fn command_struct_digest_depends_on_program() {
        let a = command("echo", &["hi"]);
        let b = command("cat", &["hi"]);
        assert_ne!(cas::digest(&a), cas::digest(&b));
    }

    #[test]
    fn command_struct_digest_depends_on_args() {
        let a = command("echo", &["a"]);
        let b = command("echo", &["b"]);
        assert_ne!(cas::digest(&a), cas::digest(&b));
    }

    #[test]
    fn command_struct_digest_depends_on_env() {
        let mut a = Command::new("run");
        a.env("KEY", "a");
        let mut b = Command::new("run");
        b.env("KEY", "b");
        assert_ne!(cas::digest(&a), cas::digest(&b));
    }

    #[test]
    fn command_digest_is_deterministic() {
        let command = digest(b"command");
        let a = Function::command(command);
        let b = Function::command(command);
        assert_eq!(cas::digest(&a), cas::digest(&b));
    }

    #[test]
    fn command_digest_changes_with_command() {
        let a = Function::command(digest(b"command-a"));
        let b = Function::command(digest(b"command-b"));
        assert_ne!(cas::digest(&a), cas::digest(&b));
    }

    #[test]
    fn function_digest_is_deterministic() {
        let command = digest(b"command");
        let config = digest(b"config");
        let a = Function::map(command, config);
        let b = Function::map(command, config);
        assert_eq!(cas::digest(&a), cas::digest(&b));
    }

    #[test]
    fn function_digest_changes_with_command() {
        let config = digest(b"config");
        let a = Function::map(digest(b"command-a"), config);
        let b = Function::map(digest(b"command-b"), config);
        assert_ne!(cas::digest(&a), cas::digest(&b));
    }

    #[test]
    fn function_digest_changes_with_config() {
        let command = digest(b"command");
        let a = Function::map(command, digest(b"config-a"));
        let b = Function::map(command, digest(b"config-b"));
        assert_ne!(cas::digest(&a), cas::digest(&b));
    }

    #[test]
    fn function_digest_changes_with_mode() {
        let command = digest(b"command");
        let config = digest(b"config");
        let map = Function::map(command, config);
        let reduce = Function::reduce(command, config);
        assert_ne!(cas::digest(&map), cas::digest(&reduce));
    }

    #[test]
    fn command_and_map_do_not_collide_on_the_same_command() {
        let command = digest(b"command");
        let a = Function::command(command);
        let b = Function::map(command, digest(b"config"));
        assert_ne!(cas::digest(&a), cas::digest(&b));
    }

    #[tokio::test]
    async fn command_variant_and_its_command_resolve_from_store() {
        let (_dir, store) = store();

        // The command to run, once, directly.
        let mut command = Command::new("python3");
        command.arg("main.py");
        let command_digest = store.put(&command).await.unwrap();

        let function = Function::command(command_digest);
        let function_digest = store.put(&function).await.unwrap();

        // Read the whole graph back out of the store using only the
        // function's digest. Input isn't part of this graph at all: it's
        // supplied separately, at call time, by whoever runs this.
        let resolved_function: Function = store.get(&function_digest).await.unwrap().unwrap();
        assert_eq!(resolved_function, function);

        let resolved_command_digest = match resolved_function {
            Function::Command(command) => command,
            _ => panic!("expected Command"),
        };
        let resolved_command: Command = store.get(&resolved_command_digest).await.unwrap().unwrap();
        assert_eq!(resolved_command, command);
    }

    #[tokio::test]
    async fn map_variant_and_its_command_and_config_resolve_from_store() {
        use crate::blob::{
            Node,
            Tree,
        };

        let (_dir, store) = store();

        // The command to run as the persistent process.
        let mut command = Command::new("serve");
        command.arg("--stdio");
        let command_digest = store.put(&command).await.unwrap();

        // The config: a Tree with one file entry, itself resolvable from
        // the store.
        let file_digest = store.put(&cas::Bytes::from_static(b"port: 8080")).await.unwrap();
        let config = Tree::new([("config.yaml".to_string(), Node::Blob(file_digest))], []);
        let config_digest = store.put(&config).await.unwrap();

        // The function tying command and config together, callable Map-style.
        let function = Function::map(command_digest, config_digest);
        let function_digest = store.put(&function).await.unwrap();

        // Read the whole graph back out of the store using only the
        // function's digest -- the resolution a caller would do before
        // invoking it.
        let resolved_function: Function = store.get(&function_digest).await.unwrap().unwrap();
        assert_eq!(resolved_function, function);

        let (resolved_command_digest, resolved_config_digest) = match resolved_function {
            Function::Map { command, config } => (command, config),
            _ => panic!("expected Map"),
        };

        let resolved_command: Command = store.get(&resolved_command_digest).await.unwrap().unwrap();
        assert_eq!(resolved_command, command);

        let resolved_config: Tree = store.get(&resolved_config_digest).await.unwrap().unwrap();
        assert_eq!(resolved_config, config);

        // The file the config tree references is itself resolvable.
        assert_eq!(
            store.get(&file_digest).await.unwrap(),
            Some(cas::Bytes::from_static(b"port: 8080"))
        );
    }
}
