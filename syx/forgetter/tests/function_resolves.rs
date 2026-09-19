//! A `Function` and everything it references resolve back out of a
//! `forgetter::Forgetter`, using only its own digest.

use content_addressing as cas;

mod common;
use common::{
    Command,
    Function,
    Node,
    Tree,
    temp_forgetter,
};

#[tokio::test]
async fn action_variant_and_its_command_and_config_resolve_from_forgetter() {
    let (_dir, forgetter) = temp_forgetter().await;

    // The command to run, once, directly.
    let command = Command::new("python3").arg("main.py");
    let command_digest = forgetter.cas().put(&command).await.unwrap();

    // The config: a Tree with one file entry, itself resolvable from
    // the forgetter.
    let file_digest =
        forgetter.cas().put(&cas::Bytes::from_static(b"threshold: 10")).await.unwrap();
    let config = Tree::new([("config.yaml".to_string(), Node::Blob(file_digest))], []);
    let config_digest = forgetter.cas().put(&config).await.unwrap();

    let function = Function::action(command_digest, config_digest);
    let function_digest = forgetter.cas().put(&function).await.unwrap();

    // Read the whole graph back out using only the function's digest.
    // Input isn't part of this graph at all: it's supplied separately,
    // at call time, by whoever runs this.
    let resolved_function: Function = forgetter.cas().get(&function_digest).await.unwrap().unwrap();
    assert_eq!(resolved_function, function);

    let (resolved_command_digest, resolved_config_digest) = match resolved_function {
        Function::Action { command, config } => (command, config),
        _ => panic!("expected Action"),
    };

    let resolved_command: Command =
        forgetter.cas().get(&resolved_command_digest).await.unwrap().unwrap();
    assert_eq!(resolved_command, command);

    let resolved_config: Tree =
        forgetter.cas().get(&resolved_config_digest).await.unwrap().unwrap();
    assert_eq!(resolved_config, config);

    // The file the config tree references is itself resolvable.
    assert_eq!(
        forgetter.cas().get(&file_digest).await.unwrap(),
        Some(cas::Bytes::from_static(b"threshold: 10"))
    );
}

#[tokio::test]
async fn server_variant_and_its_command_and_config_resolve_from_forgetter() {
    let (_dir, forgetter) = temp_forgetter().await;

    // The command to run as the persistent process.
    let command = Command::new("serve").arg("--config");
    let command_digest = forgetter.cas().put(&command).await.unwrap();

    // The config: a Tree with one file entry, itself resolvable from
    // the forgetter.
    let file_digest = forgetter.cas().put(&cas::Bytes::from_static(b"port: 8080")).await.unwrap();
    let config = Tree::new([("config.yaml".to_string(), Node::Blob(file_digest))], []);
    let config_digest = forgetter.cas().put(&config).await.unwrap();

    // The function tying command and config together, callable as a server.
    let function = Function::server(command_digest, config_digest);
    let function_digest = forgetter.cas().put(&function).await.unwrap();

    // Read the whole graph back out using only the function's digest,
    // the resolution a caller would do before invoking it.
    let resolved_function: Function = forgetter.cas().get(&function_digest).await.unwrap().unwrap();
    assert_eq!(resolved_function, function);

    let (resolved_command_digest, resolved_config_digest) = match resolved_function {
        Function::Server { command, config } => (command, config),
        _ => panic!("expected Server"),
    };

    let resolved_command: Command =
        forgetter.cas().get(&resolved_command_digest).await.unwrap().unwrap();
    assert_eq!(resolved_command, command);

    let resolved_config: Tree =
        forgetter.cas().get(&resolved_config_digest).await.unwrap().unwrap();
    assert_eq!(resolved_config, config);

    // The file the config tree references is itself resolvable.
    assert_eq!(
        forgetter.cas().get(&file_digest).await.unwrap(),
        Some(cas::Bytes::from_static(b"port: 8080"))
    );
}
