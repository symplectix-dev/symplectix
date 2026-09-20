//! A digest that was never stored is reported as absent, not an error.

use content_addressing::Bytes;

mod common;
use common::temp_store;

#[tokio::test]
async fn get_missing_digest_is_none() {
    let (_dir, store) = temp_store().await;
    assert_eq!(store.get::<Bytes>(&cas_testing::digest_bytes(b"missing")).await.unwrap(), None);
}
