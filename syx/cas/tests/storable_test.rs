//! `cas` itself has no opinion on CBOR; these tests exercise
//! `ToBytes`/`FromBytes` through a type outside the crate (`Example`)
//! that builds its own encoding on top of `cas`'s public API.

mod support;

use support::Example;

#[test]
fn digest_is_deterministic() {
    let a = Example { name: "foo".to_string(), count: 1 };
    let b = Example { name: "foo".to_string(), count: 1 };
    assert_eq!(cas::digest(&a), cas::digest(&b));
}

#[test]
fn digest_depends_on_every_field() {
    let base = Example { name: "foo".to_string(), count: 1 };
    let other_name = Example { name: "bar".to_string(), count: 1 };
    let other_count = Example { name: "foo".to_string(), count: 2 };
    assert_ne!(cas::digest(&base), cas::digest(&other_name));
    assert_ne!(cas::digest(&base), cas::digest(&other_count));
}

#[test]
fn digest_byte_buf() {
    let mut h = cas::Hasher::new();
    h.part(b"hello");
    let d = h.digest();

    let d_bytes = serde_bytes::ByteBuf::from(d.as_ref());
    assert_eq!(cbor2::to_canonical_vec(&d_bytes).unwrap(), cbor2::to_canonical_vec(&d).unwrap());
}

#[tokio::test]
async fn get_returns_an_error_for_corrupted_content_instead_of_panicking() {
    use std::os::unix::fs::PermissionsExt;

    let dir = testing::tempdir();
    let store = cas::Store::create(dir.path()).unwrap();
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
