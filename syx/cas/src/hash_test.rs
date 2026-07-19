use super::*;

#[tokio::test]
async fn async_reader_is_chunked_not_buffered_at_once() {
    // Content larger than one BUF_SIZE read still hashes correctly,
    // proving the reader loops instead of assuming a single read call
    // drains everything.
    let content = vec![0x42u8; BUF_SIZE * 2 + 1];
    let mut h = Hasher::new();
    h.read_from(content.len() as u64, io::Cursor::new(&content)).await.unwrap();

    let mut want = Hasher::new();
    want.part(&content);
    assert_eq!(h.digest(), want.digest());
}
