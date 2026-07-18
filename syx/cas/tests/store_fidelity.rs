//! Content stored in a `Store` is preserved faithfully.

mod support;
use support::{
    digest_bytes,
    store,
};

#[tokio::test]
async fn content_put_is_returned_unchanged_by_get() {
    let (_dir, store) = store();
    let d = store.put(&cas::Bytes::from_static(b"hello")).await.unwrap();
    assert!(store.path(&d).exists());
    assert_eq!(store.get(&d).await.unwrap(), Some(cas::Bytes::from_static(b"hello")));
}

#[tokio::test]
async fn copy_from_file_accepts_a_file_and_streams_it_in() {
    // A file already on disk (not just in-memory bytes) can be
    // ingested via copy_from_file, streamed in without requiring the
    // caller to load it into memory first.
    let (_dir, store) = store();
    let src_dir = testing::tempdir();
    let src = src_dir.path().join("blob");
    std::fs::write(&src, b"hello").unwrap();

    let mut file = tokio::fs::File::open(&src).await.unwrap();
    let len = file.metadata().await.unwrap().len();
    let d = store.copy_from_file(len, &mut file).await.unwrap();
    assert_eq!(d, digest_bytes(b"hello"));
    assert_eq!(store.get(&d).await.unwrap(), Some(cas::Bytes::from_static(b"hello")));
}

#[tokio::test]
async fn path_points_at_the_stored_file() {
    let (dir, store) = store();
    let d = store.put(&cas::Bytes::from_static(b"hello")).await.unwrap();

    let path = store.path(&d);
    assert_eq!(path, dir.path().join(d.hex(2)));
    assert_eq!(std::fs::read(path).unwrap(), b"hello".to_vec());
}
