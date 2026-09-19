//! A digest that was never stored is reported as absent, not an error.

use content_addressing as cas;

mod common;
use common::temp_forgetter;

#[tokio::test]
async fn get_missing_digest_is_none() {
    let (_dir, forgetter) = temp_forgetter().await;
    assert_eq!(
        forgetter.cas().get::<cas::Bytes>(&cas_testing::digest_bytes(b"missing")).await.unwrap(),
        None
    );
}
