use std::sync::Arc;

use bytes::{
    BufMut,
    Bytes,
};
use content_addressing::{
    Chunking,
    Codec,
    ContentFlags,
    Digest,
    Hasher,
};
use object_store::ObjectStore;
use object_store::local::LocalFileSystem;
use object_store::memory::InMemory;

use super::*;

fn in_memory() -> Arc<dyn ObjectStore> {
    Arc::new(InMemory::new())
}

/// A `LocalFileSystem` backend, paired with the `TempDir` it's rooted at.
/// Callers must keep the `TempDir` alive for as long as the backend is used.
fn local_fs() -> (testing::TempDir, Arc<dyn ObjectStore>) {
    let tmp = testing::tempdir();
    let backend = Arc::new(LocalFileSystem::new_with_prefix(tmp.path()).unwrap());
    (tmp, backend)
}

/// A `Storage` to drive tests against: a fresh in-memory `slatedb::Db`,
/// packs written to `blobs_backend`, and blobs staged in a `Logger`
/// rooted at a fresh local `TempDir`, which this keeps alive alongside
/// it. No test overrides `cas_prefix`/chunking/encoding, so those just
/// use their defaults.
struct Env {
    _logger_dir: testing::TempDir,
    storage:     Storage,
}

impl Env {
    async fn with_threshold(blobs_backend: Arc<dyn ObjectStore>, threshold: u64) -> Self {
        let db = slatedb::Db::builder("test", in_memory()).build().await.unwrap();
        let logger_dir = testing::tempdir();
        let (logger, mut replayed) =
            Logger::open(logger_dir.path(), u16::MAX, threshold, None).await.unwrap();
        assert!(replayed.next().is_none());
        let storage = Storage::new(
            Arc::new(logger),
            Arc::new(KeyDir::rebuild(replayed, Codec::new()).await),
            db,
            blobs_backend,
            Flushing::new(),
            Arc::from(DEFAULT_CAS_PREFIX),
            Chunking::new(),
            Codec::new(),
        );
        Self { _logger_dir: logger_dir, storage }
    }

    async fn new(blobs_backend: Arc<dyn ObjectStore>) -> Self {
        Self::with_threshold(blobs_backend, DEFAULT_FLUSH_THRESHOLD).await
    }

    fn cas(&self) -> &Storage {
        &self.storage
    }
}

/// Some chunk digest referenced by `exclude`'s own manifest, other than
/// `exclude` itself, for tests that need an existing chunk key to target
/// for corruption without independently recomputing chunk digests.
async fn any_key_except(cas: &Storage, exclude: Digest) -> Digest {
    let (_, manifest_bytes) = cas.load(&exclude).await.unwrap().expect("manifest present");
    let manifest = decode_chunks(&manifest_bytes).unwrap();
    manifest
        .iter()
        .map(|e| e.digest)
        .find(|d| *d != exclude)
        .expect("multi-chunk content should store more than just the manifest")
}

fn encode(flags: ContentFlags, raw: Vec<u8>) -> Vec<u8> {
    Codec::new().encode(flags, raw)
}

/// Forces `logger`'s active segment out, then packs everything
/// currently pending. `Storage::flush_pending` deliberately doesn't
/// rotate the active segment itself, so tests that want a deterministic
/// "everything staged so far is now packed" use this instead of calling
/// `flush_pending` alone.
async fn flush(cas: &Storage) {
    let _ = cas.logger.rotate().await;
    cas.flush_pending().await.unwrap();
}

#[tokio::test]
async fn a_single_chunks_digest_is_the_content_digest_not_a_wrapped_one() {
    // This is what makes a small standalone blob dedup against the
    // same content appearing as one chunk inside a larger blob: both
    // are keyed by the exact same digest. Runs against both inner
    // object stores, since this is a property of this module's own
    // digest scheme, not of whichever store happens to be holding the
    // packs.
    async fn check(cas: &Storage) {
        let content = testing::random_bytes(4096); // well under CHUNK_MIN_SIZE
        let content_digest = Hasher::new().part(&content).digest();
        let d = cas.put(&Bytes::from(content)).await.unwrap();
        assert_eq!(d, content_digest);
    }

    check(Env::new(in_memory()).await.cas()).await;
    let (_tmp, inner) = local_fs();
    check(Env::new(inner).await.cas()).await;
}

#[tokio::test]
async fn identical_chunks_across_different_blobs_are_stored_once() {
    // Long enough, and shared for long enough, that content-defined
    // chunking is guaranteed to produce at least one identical cut
    // chunk in both blobs before they diverge.
    let shared = testing::random_bytes(Chunking::MAX_SIZE * 2);
    let blob_a = {
        let mut blob_a = shared.clone();
        blob_a.extend_from_slice(b"-a-suffix");
        Bytes::from(blob_a)
    };
    let blob_b = {
        let mut blob_b = shared;
        blob_b.extend_from_slice(b"-b-suffix");
        Bytes::from(blob_b)
    };

    // How many keys after putting `blob` and flushing it into packs.
    async fn count_keys(cas: &Storage, blob: Bytes) -> usize {
        cas.put(&blob).await.unwrap();
        flush(cas).await;
        cas.entry_count().await.unwrap()
    }

    async fn check(cas: &Storage, blob_a: &Bytes, blob_b: &Bytes, baseline: usize) {
        cas.put(blob_a).await.unwrap();
        flush(cas).await;
        let count_before = cas.entry_count().await.unwrap();
        cas.put(blob_b).await.unwrap();
        flush(cas).await;
        let count_after = cas.entry_count().await.unwrap();

        let new_keys = count_after - count_before;
        assert!(
            new_keys < baseline,
            "storing blob_b needed {new_keys} new keys, expected fewer than the {baseline} \
            it needs alone, since blob_a already stored the chunks they share"
        );
    }

    let mem_keys = count_keys(Env::new(in_memory()).await.cas(), blob_b.clone()).await;
    let (_tmp, inner) = local_fs();
    let tmp_keys = count_keys(Env::new(inner).await.cas(), blob_b.clone()).await;
    // The baseline is a property of blob_b's content and this module's
    // chunking, not of which backend computed it.
    assert_eq!(mem_keys, tmp_keys);

    check(Env::new(in_memory()).await.cas(), &blob_a, &blob_b, mem_keys).await;
    let (_tmp, inner) = local_fs();
    check(Env::new(inner).await.cas(), &blob_a, &blob_b, tmp_keys).await;
}

#[tokio::test]
async fn flush_pending_moves_a_staged_entry_out_of_the_logger_and_into_a_pack() {
    let env = Env::with_threshold(in_memory(), 1024 * 1024).await;
    let cas = env.cas();

    let content = Bytes::from_static(b"0123456789");
    let d = cas.put(&content).await.unwrap();
    assert!(cas.staged.contains(d));
    assert!(cas.get_entry(d).await.unwrap().is_none());

    flush(cas).await;
    assert!(!cas.staged.contains(d));
    assert!(cas.get_entry(d).await.unwrap().is_some());

    assert_eq!(cas.get::<Bytes>(&d).await.unwrap(), Some(content));
}

#[tokio::test]
async fn get_returns_invalid_data_for_tampered_content() {
    async fn check(cas: &Storage) {
        let d = cas.put(&Bytes::from_static(b"hello")).await.unwrap();
        flush(cas).await;

        // Overwrite the stored bytes with content that doesn't hash
        // back to `d`, simulating corruption.
        let tampered = encode(ContentFlags::empty(), b"not hello".to_vec());
        cas.put_blob(d, Bytes::from(tampered)).await.unwrap();
        flush(cas).await;

        let err = cas.get::<Bytes>(&d).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    check(Env::new(in_memory()).await.cas()).await;
    let (_tmp, inner) = local_fs();
    check(Env::new(inner).await.cas()).await;
}

#[tokio::test]
async fn get_returns_invalid_data_for_a_tampered_chunk() {
    // Needs a real (non-manifest) key to target, so this one stays
    // `in_memory`-only rather than being generalized over the backend.
    let env = Env::new(in_memory()).await;
    let cas = env.cas();
    let content = testing::random_bytes(Chunking::MAX_SIZE * 2);
    let d = cas.put(&Bytes::from(content)).await.unwrap();
    flush(cas).await;

    let chunk_key = any_key_except(cas, d).await;
    let tampered = encode(ContentFlags::empty(), b"tampered chunk content".to_vec());
    cas.put_blob(chunk_key, Bytes::from(tampered)).await.unwrap();
    flush(cas).await;

    let err = cas.get::<Bytes>(&d).await.unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

#[tokio::test]
async fn read_into_returns_invalid_data_for_tampered_content() {
    async fn check(cas: &Storage) {
        let d = cas.put(&Bytes::from_static(b"hello")).await.unwrap();
        flush(cas).await;

        let tampered = encode(ContentFlags::empty(), b"not hello".to_vec());
        cas.put_blob(d, Bytes::from(tampered)).await.unwrap();
        flush(cas).await;

        let mut out = Vec::new();
        let err = cas.read_into(&d, &mut out).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    check(Env::new(in_memory()).await.cas()).await;
    let (_tmp, inner) = local_fs();
    check(Env::new(inner).await.cas()).await;
}

#[tokio::test]
async fn read_into_returns_invalid_data_for_a_tampered_chunk() {
    // Needs a real (non-manifest) key to target, so this one stays
    // `in_memory`-only rather than being generalized over the backend.
    let env = Env::new(in_memory()).await;
    let cas = env.cas();
    let content = testing::random_bytes(Chunking::MAX_SIZE * 2);
    let d = cas.put(&Bytes::from(content)).await.unwrap();
    flush(cas).await;

    let chunk_key = any_key_except(cas, d).await;
    let tampered = encode(ContentFlags::empty(), b"tampered chunk content".to_vec());
    cas.put_blob(chunk_key, Bytes::from(tampered)).await.unwrap();
    flush(cas).await;

    let mut out = Vec::new();
    let err = cas.read_into(&d, &mut out).await.unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

#[tokio::test]
async fn get_returns_invalid_data_for_a_tampered_manifest() {
    async fn check(cas: &Storage) {
        let content = testing::random_bytes(Chunking::MAX_SIZE * 2);
        let d = cas.put(&Bytes::from(content)).await.unwrap();
        flush(cas).await;

        let tampered = encode(ContentFlags::CHUNKED, b"not a valid manifest body".to_vec());
        cas.put_blob(d, Bytes::from(tampered)).await.unwrap();
        flush(cas).await;

        let err = cas.get::<Bytes>(&d).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    check(Env::new(in_memory()).await.cas()).await;
    let (_tmp, inner) = local_fs();
    check(Env::new(inner).await.cas()).await;
}

#[tokio::test]
async fn get_returns_invalid_data_when_manifest_references_a_missing_chunk() {
    async fn check(cas: &Storage) {
        let (present_digest, present_raw) =
            (Hasher::new().part(b"present").digest(), b"present".to_vec());
        cas.put_blob(
            present_digest,
            Bytes::from(encode(ContentFlags::empty(), present_raw.clone())),
        )
        .await
        .unwrap();
        let missing_digest = Hasher::new().part(b"never written").digest();

        let mut manifest = Vec::new();
        manifest.put_slice(present_digest.as_ref());
        manifest.put_u32(present_raw.len() as u32);
        manifest.put_slice(missing_digest.as_ref());
        manifest.put_u32(13);

        let blob_digest = {
            let mut h = Hasher::new();
            h.parts([present_digest.as_ref(), missing_digest.as_ref()]);
            h.digest()
        };
        cas.put_blob(blob_digest, Bytes::from(encode(ContentFlags::CHUNKED, manifest)))
            .await
            .unwrap();
        flush(cas).await;

        let err = cas.get::<Bytes>(&blob_digest).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    check(Env::new(in_memory()).await.cas()).await;
    let (_tmp, inner) = local_fs();
    check(Env::new(inner).await.cas()).await;
}
