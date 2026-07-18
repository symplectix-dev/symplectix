//! A `Store`'s behavior when stored content is corrupted on disk.

mod common;
use common::{
    Example,
    store,
};

#[tokio::test]
async fn get_returns_an_error_for_corrupted_content_instead_of_panicking() {
    use std::os::unix::fs::PermissionsExt;

    let (dir, store) = store();
    let d = store.put(&Example { name: "foo".to_string(), count: 1 }).await.unwrap();

    // Overwrite the stored bytes on disk with something that isn't
    // valid CBOR at all, simulating on-disk corruption.
    let path = dir.path().join(d.hex(2));
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(&path, perms).unwrap();
    std::fs::write(&path, b"not cbor").unwrap();

    let err = store.get::<Example>(&d).await.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
}
