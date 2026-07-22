//! A `Function` and everything it references resolve back out of a
//! `ply::Store`, using only its own digest.

mod common;
use common::store;

#[tokio::test]
async fn command_variant_and_its_command_resolve_from_store() {
    let (_dir, store) = store();

    // The command to run, once, directly.
    let mut command = func::Command::new("python3");
    command.arg("main.py");
    let command_digest = store.put(&command).await.unwrap();

    let function = func::Function::command(command_digest);
    let function_digest = store.put(&function).await.unwrap();

    // Read the whole graph back out of the store using only the
    // function's digest. Input isn't part of this graph at all: it's
    // supplied separately, at call time, by whoever runs this.
    let resolved_function: func::Function = store.get(&function_digest).await.unwrap().unwrap();
    assert_eq!(resolved_function, function);

    let resolved_command_digest = match resolved_function {
        func::Function::Command(command) => command,
        _ => panic!("expected Command"),
    };
    let resolved_command: func::Command =
        store.get(&resolved_command_digest).await.unwrap().unwrap();
    assert_eq!(resolved_command, command);
}

#[tokio::test]
async fn map_variant_and_its_command_and_config_resolve_from_store() {
    let (_dir, store) = store();

    // The command to run as the persistent process.
    let mut command = func::Command::new("serve");
    command.arg("--stdio");
    let command_digest = store.put(&command).await.unwrap();

    // The config: a Tree with one file entry, itself resolvable from
    // the store.
    let file_digest = store.put(&cas::Bytes::from_static(b"port: 8080")).await.unwrap();
    let config = ply::Tree::new([("config.yaml".to_string(), ply::Node::Blob(file_digest))], []);
    let config_digest = store.put(&config).await.unwrap();

    // The function tying command and config together, callable Map-style.
    let function = func::Function::map(command_digest, config_digest);
    let function_digest = store.put(&function).await.unwrap();

    // Read the whole graph back out of the store using only the
    // function's digest -- the resolution a caller would do before
    // invoking it.
    let resolved_function: func::Function = store.get(&function_digest).await.unwrap().unwrap();
    assert_eq!(resolved_function, function);

    let (resolved_command_digest, resolved_config_digest) = match resolved_function {
        func::Function::Map { command, config } => (command, config),
        _ => panic!("expected Map"),
    };

    let resolved_command: func::Command =
        store.get(&resolved_command_digest).await.unwrap().unwrap();
    assert_eq!(resolved_command, command);

    let resolved_config: ply::Tree = store.get(&resolved_config_digest).await.unwrap().unwrap();
    assert_eq!(resolved_config, config);

    // The file the config tree references is itself resolvable.
    assert_eq!(
        store.get(&file_digest).await.unwrap(),
        Some(cas::Bytes::from_static(b"port: 8080"))
    );
}
