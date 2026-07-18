//! A digest that was never stored is reported as absent, not an error.

mod common;
use common::{
    digest_bytes,
    store,
};

#[tokio::test]
async fn missing_digest_does_not_exist() {
    let (_dir, store) = store();
    assert!(!store.path(&digest_bytes(b"hello")).exists());
}

#[tokio::test]
async fn get_missing_digest_is_none() {
    let (_dir, store) = store();
    assert_eq!(store.get::<cas::Bytes>(&digest_bytes(b"missing")).await.unwrap(), None);
}
