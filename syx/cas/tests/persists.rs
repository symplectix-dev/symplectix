//! What a `Storage` persists across instances.

use content_addressing::Bytes;

mod common;

#[tokio::test]
async fn content_persists_across_store_instances() {
    // A fresh Storage instance over the same root sees content a
    // prior instance wrote: proof it actually landed in the backing
    // store.
    let dir = testing::tempdir();

    let writer = common::store(dir.path()).await;
    let d = writer.put(&Bytes::from_static(b"hello")).await.unwrap();
    // Drop `writer` before reopening the same root, so the two instances
    // stay sequential rather than coexisting live.
    drop(writer);

    let reader = common::store(dir.path()).await;
    assert_eq!(reader.get(&d).await.unwrap(), Some(Bytes::from_static(b"hello")));
}
