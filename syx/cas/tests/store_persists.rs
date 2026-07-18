//! What a `Store` persists to disk.

mod support;
use support::store;

#[tokio::test]
async fn content_persists_across_store_instances() {
    // A fresh Store instance over the same root sees content a prior
    // instance wrote: proof it actually landed on disk.
    let dir = testing::tempdir();

    let writer = cas::Store::open(dir.path()).unwrap();
    let d = writer.put(&cas::Bytes::from_static(b"hello")).await.unwrap();

    let reader = cas::Store::open(dir.path()).unwrap();
    assert!(reader.path(&d).exists());
    assert_eq!(reader.get(&d).await.unwrap(), Some(cas::Bytes::from_static(b"hello")));
}

#[tokio::test]
async fn content_is_sharded_under_root() {
    let (dir, store) = store();
    let d = store.put(&cas::Bytes::from_static(b"hello")).await.unwrap();

    let path = dir.path().join(d.hex(2));
    assert_eq!(std::fs::read(path).unwrap(), b"hello".to_vec());
}

#[tokio::test]
async fn put_stores_content_read_only() {
    use std::os::unix::fs::PermissionsExt;

    let (dir, store) = store();
    let d = store.put(&cas::Bytes::from_static(b"hello")).await.unwrap();

    let path = dir.path().join(d.hex(2));
    let perms = std::fs::metadata(&path).unwrap().permissions();
    assert!(perms.readonly());
    assert_eq!(perms.mode() & 0o200, 0);
}

#[tokio::test]
async fn put_skips_writing_when_content_already_exists() {
    use std::os::unix::fs::PermissionsExt;

    let (_dir, store) = store();
    let d = store.put(&cas::Bytes::from_static(b"hello")).await.unwrap();

    // Make the destination's parent directory read-only: if the
    // second put below actually tried to persist (rename a temp
    // file, created elsewhere under `root/.tmp`, into this
    // directory), that would now fail with a permission error.
    let path = store.path(&d);
    let dir = path.parent().unwrap();
    let permissions = std::fs::metadata(dir).unwrap().permissions();
    let mut readonly = permissions.clone();
    readonly.set_mode(0o500);
    std::fs::set_permissions(dir, readonly).unwrap();

    let d2 = store.put(&cas::Bytes::from_static(b"hello")).await.unwrap();
    assert_eq!(d, d2);

    // Restore write permission so the tempdir can clean itself up.
    std::fs::set_permissions(dir, permissions).unwrap();
}
