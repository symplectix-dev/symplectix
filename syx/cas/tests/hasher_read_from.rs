//! `Hasher::read_from` behavior.

mod common;
use common::digest_bytes;

#[tokio::test]
async fn hashing_via_read_from_matches_hashing_in_memory_bytes() {
    let content = b"hello, reader".to_vec();
    let mut h = cas::Hasher::new();
    h.read_from(content.len() as u64, std::io::Cursor::new(&content)).await.unwrap();
    assert_eq!(h.digest(), digest_bytes(&content));
}

#[tokio::test]
async fn read_from_works_with_a_real_file() {
    // A real tokio::fs::File (not just an in-memory Cursor) implements
    // AsyncRead the same way; the caller supplies the length via `stat()`.
    let path = testing::rlocation("_main/.rustfmt.toml");
    let content = std::fs::read(&path).unwrap();
    let file = tokio::fs::File::open(&path).await.unwrap();
    let len = file.metadata().await.unwrap().len();

    let mut h = cas::Hasher::new();
    h.read_from(len, file).await.unwrap();

    assert_eq!(h.digest(), digest_bytes(&content));
}

#[tokio::test]
async fn read_from_returns_an_error_when_the_reader_ends_early() {
    let content = b"short".to_vec();
    let mut h = cas::Hasher::new();
    assert!(h.read_from(content.len() as u64 + 1, std::io::Cursor::new(&content)).await.is_err());
}
