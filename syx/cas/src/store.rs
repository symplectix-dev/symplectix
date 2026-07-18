//! A content-addressed blob store.
use std::future::{
    self,
    Future,
};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use bytes::Bytes;
use moka::future::Cache;
use tokio::io::{
    AsyncRead,
    AsyncSeek,
    AsyncSeekExt as _,
};
use tokio::{
    fs,
    task,
};

use crate::hash::{
    Digest,
    FromBytes,
    Hasher,
    ToBytes,
};

/// A content-addressed store mapping `Digest` keys to blob content: files
/// under `root`, sharded the same way as `Digest::hex`.
///
/// TODO: no GC yet. The likely shape:
/// When given a set of digests to delete, a digest not in `cache` has
/// no recent write activity, so delete it right away; a digest that is
/// in `cache` gets recorded in a separate pending-delete marker instead,
/// and `cache`'s own `eviction_listener` does the actual `fs::remove_file`
/// once `cache` naturally lets go of that digest.
///
/// Open question:
/// What happens if the same digest is `put` again while marked?
/// - delete could still win regardless
/// - the write path could clear the marker
///
/// Also need to check whether `eviction_listener` fires for entries still
/// live in `cache` when `Store` itself drops. If not, `Store` will need
/// its own `Drop` impl that synchronously deletes whatever is still in
/// the pending-delete marker at that point.
pub struct Store {
    root:  PathBuf,
    /// Coalesces concurrent write calls for the same
    /// not-yet-persisted digest into a single write.
    cache: Cache<Digest, ()>,
}

impl Store {
    const TMP: &str = ".tmp";
    const CACHE_TIME_TO_LIVE: Duration = Duration::from_secs(60);
    const CACHE_MAX_CAPACITY: u64 = 10_000;

    /// Open a store rooted at `root`, creating `root` and its `.tmp`
    /// subdirectory if they don't already exist.
    ///
    /// `persist` relies on `root/.tmp`: a rename between every digest-addressed
    /// path and `root/.tmp` must be atomic. That's checked once here, by actually
    /// renaming a probe file from `root/.tmp` into `root`.
    pub fn create(root: impl Into<PathBuf>) -> io::Result<Self> {
        let root = root.into();
        let tmp = root.join(Self::TMP);
        std::fs::create_dir_all(&tmp)?;

        let probe = tempfile::NamedTempFile::new_in(&tmp)?;
        let probe_path = root.join(".probe");
        probe.persist(&probe_path).map_err(|e| {
            io::Error::new(
                e.error.kind(),
                format!(
                    "Store requires {root} and {tmp} to be on the same filesystem, \
                    so that persisting a blob is atomic; renaming a probe file \
                    between them failed: {err}",
                    root = root.display(),
                    tmp = tmp.display(),
                    err = e.error,
                ),
            )
        })?;

        Ok(Store {
            root,
            cache: Cache::builder()
                .time_to_live(Self::CACHE_TIME_TO_LIVE)
                .max_capacity(Self::CACHE_MAX_CAPACITY)
                .build(),
        })
    }

    /// The path content addressed by `digest` whether or not
    /// it's actually been stored yet. For a caller that wants
    /// to work with the file directly (e.g. mmap it)
    /// instead of loading it via `get`.
    pub fn path(&self, digest: &Digest) -> PathBuf {
        self.root.join(digest.hex(2))
    }

    /// Reads the content at `digest`, if present.
    pub async fn get<T: FromBytes>(&self, digest: &Digest) -> io::Result<Option<T>> {
        let bytes = match fs::read(self.path(digest)).await {
            Ok(bytes) => Bytes::from(bytes),
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };

        let content = T::from_bytes(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e:?}")))?;

        Ok(Some(content))
    }

    /// Store `content`, addressed by its own digest, and return that
    /// digest.
    pub async fn put<T: ToBytes>(&self, content: &T) -> io::Result<Digest> {
        let bytes =
            content.to_bytes().unwrap_or_else(|_| panic!("serializing to bytes should not fail"));

        let digest = {
            let mut h = Hasher::new();
            h.part(&bytes);
            h.digest()
        };

        self.persist_bytes(digest, &bytes).await?;
        Ok(digest)
    }

    /// Store the content read from `r` of `len` bytes,
    /// addressed by its own digest.
    ///
    /// If `len` is small enough, `r` is hashed straight into memory and
    /// handled like `put`. Otherwise, hashing and writing `r` to a temp file
    /// happen in the same pass. This means that, unlike `copy_from_file`,
    /// an already-stored duplicate still costs one temp-file write.
    pub async fn copy_from<R>(&self, len: u64, r: &mut R) -> io::Result<Digest>
    where
        R: AsyncRead + Unpin,
    {
        const INLINE_MAX_BYTES: u64 = 64 * 1024;

        if len <= INLINE_MAX_BYTES {
            let mut bytes = Vec::with_capacity(len as usize);
            let digest = {
                let mut h = Hasher::new();
                h.tee_read_from(len, r, &mut bytes).await?;
                h.digest()
            };
            self.persist_bytes(digest, &bytes).await?;
            return Ok(digest);
        }

        let tmp = self.new_temp_file().await?;
        let mut file = fs::File::from_std(tmp.as_file().try_clone()?);

        let digest = {
            let mut h = Hasher::new();
            h.tee_read_from(len, r, &mut file).await?;
            h.digest()
        };

        self.persist_file(digest, future::ready(Ok(tmp))).await?;
        Ok(digest)
    }

    /// Store the content read from a seekable `r` of `len` bytes.
    ///
    /// Reads `r` once to compute the digest; only if that digest isn't already
    /// stored, seek back to the start and copy `r` in.
    pub async fn copy_from_file<R>(&self, len: u64, r: &mut R) -> io::Result<Digest>
    where
        R: AsyncRead + AsyncSeek + Unpin,
    {
        let digest = {
            let mut h = Hasher::new();
            h.read_from(len, &mut *r).await?;
            h.digest()
        };

        self.persist_file(digest, async {
            r.rewind().await?;
            let tmp = self.new_temp_file().await?;
            let mut file = fs::File::from_std(tmp.as_file().try_clone()?);
            tokio::io::copy(r, &mut file).await?;
            Ok(tmp)
        })
        .await?;

        Ok(digest)
    }

    /// A fresh, empty temp file under `root/.tmp`.
    async fn new_temp_file(&self) -> io::Result<tempfile::NamedTempFile> {
        let dir = self.root.join(Self::TMP);
        task::spawn_blocking(move || tempfile::NamedTempFile::new_in(dir))
            .await
            .expect("creating the temp file should not panic")
    }

    /// Persist already-in-memory `bytes` at `digest` via a temp file,
    /// coalescing concurrent writes for the same digest.
    async fn persist_bytes<T>(&self, digest: Digest, bytes: &T) -> io::Result<()>
    where
        T: AsRef<[u8]>,
    {
        self.persist_file(digest, async {
            let tmp = self.new_temp_file().await?;
            fs::write(tmp.path(), bytes.as_ref()).await?;
            Ok(tmp)
        })
        .await
    }

    /// Persist a `file` at `digest`.
    async fn persist_file<F>(&self, digest: Digest, file: F) -> io::Result<()>
    where
        F: Future<Output = io::Result<tempfile::NamedTempFile>>,
    {
        let dst = self.path(&digest);
        self.cache
            .try_get_with(digest, async move {
                // Not `self.exists`: `dst` is already known here, so
                // check it directly instead of recomputing it.
                if fs::try_exists(&dst).await? {
                    return Ok(());
                }
                let src = file.await?;
                self.persist(src, dst).await
            })
            .await
            .map_err(|e| io::Error::new(e.kind(), e.to_string()))
    }

    /// Make `tmp` read-only and rename it into place at `dst`.
    ///
    /// This is best-effort, not a concurrency-safe write-once: two
    /// writers racing on the same digest may both persist and the last
    /// one wins. That is harmless here because `dst` is a deterministic
    /// function of the digest, so whichever write wins holds the
    /// intended content.
    async fn persist(&self, tmp: tempfile::NamedTempFile, dst: PathBuf) -> io::Result<()> {
        let dir =
            dst.parent().expect("path is always root/xx/xx/rest, so it has a parent").to_owned();
        fs::create_dir_all(&dir).await?;

        task::spawn_blocking(move || -> io::Result<()> {
            // Read-only: a digest is only valid as long as the bytes it
            // was computed from never change, so CAS entries must not be
            // mutable once written.
            let mut perms = tmp.as_file().metadata()?.permissions();
            perms.set_readonly(true);
            tmp.as_file().set_permissions(perms)?;
            tmp.persist(dst)?;
            Ok(())
        })
        .await
        .expect("persisting the temp file should not panic")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Store {
        /// Whether content is already stored at `digest`, on disk.
        pub async fn exists(&self, digest: &Digest) -> io::Result<bool> {
            fs::try_exists(self.path(digest)).await
        }
    }

    fn digest(bytes: &[u8]) -> Digest {
        let mut h = Hasher::new();
        h.part(bytes);
        h.digest()
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Example {
        value: u32,
    }

    impl ToBytes for Example {
        type Error = cbor2::ser::Error;

        fn to_bytes(&self) -> Result<Bytes, Self::Error> {
            cbor2::to_canonical_vec(self).map(Bytes::from)
        }
    }

    impl FromBytes for Example {
        type Error = cbor2::de::Error;

        fn from_bytes(bytes: Bytes) -> Result<Self, Self::Error> {
            cbor2::from_slice(&bytes)
        }
    }

    fn store() -> (testing::TempDir, Store) {
        let dir = testing::tempdir();
        let store = Store::create(dir.path()).unwrap();
        (dir, store)
    }

    #[tokio::test]
    async fn missing_digest_does_not_exist() {
        let (_dir, store) = store();
        assert!(!store.exists(&digest(b"hello")).await.unwrap());
    }

    #[tokio::test]
    async fn get_missing_digest_is_none() {
        let (_dir, store) = store();
        assert_eq!(store.get::<Bytes>(&digest(b"missing")).await.unwrap(), None);
    }

    #[tokio::test]
    async fn put_then_get() {
        let (_dir, store) = store();
        let d = store.put(&Bytes::from_static(b"hello")).await.unwrap();
        assert!(store.exists(&d).await.unwrap());
        assert_eq!(store.get(&d).await.unwrap(), Some(Bytes::from_static(b"hello")));
    }

    #[tokio::test]
    async fn put_returns_the_content_digest() {
        let (_dir, store) = store();
        let d = store.put(&Bytes::from_static(b"hello")).await.unwrap();
        assert_eq!(d, digest(b"hello"));
    }

    #[tokio::test]
    async fn put_skips_writing_when_content_already_exists() {
        use std::os::unix::fs::PermissionsExt;

        let (_dir, store) = store();
        let d = store.put(&Bytes::from_static(b"hello")).await.unwrap();

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

        let d2 = store.put(&Bytes::from_static(b"hello")).await.unwrap();
        assert_eq!(d, d2);

        // Restore write permission so the tempdir can clean itself up.
        std::fs::set_permissions(dir, permissions).unwrap();
    }

    #[tokio::test]
    async fn content_persists_across_store_instances() {
        // A fresh Store instance over the same root sees content a prior
        // instance wrote: proof it actually landed on disk.
        let dir = testing::tempdir();

        let writer = Store::create(dir.path()).unwrap();
        let d = writer.put(&Bytes::from_static(b"hello")).await.unwrap();

        let reader = Store::create(dir.path()).unwrap();
        assert!(reader.exists(&d).await.unwrap());
        assert_eq!(reader.get(&d).await.unwrap(), Some(Bytes::from_static(b"hello")));
    }

    #[tokio::test]
    async fn content_is_sharded_under_root() {
        let (dir, store) = store();
        let d = store.put(&Bytes::from_static(b"hello")).await.unwrap();

        let path = dir.path().join(d.hex(2));
        assert_eq!(std::fs::read(path).unwrap(), b"hello".to_vec());
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

        let mut file = fs::File::open(&src).await.unwrap();
        let len = file.metadata().await.unwrap().len();
        let d = store.copy_from_file(len, &mut file).await.unwrap();
        assert_eq!(d, digest(b"hello"));
        assert_eq!(store.get(&d).await.unwrap(), Some(Bytes::from_static(b"hello")));
    }

    #[tokio::test]
    async fn copy_from_file_produces_the_same_digest_as_put() {
        let (_dir, store) = store();
        let src_dir = testing::tempdir();
        let src = src_dir.path().join("blob");
        std::fs::write(&src, b"hello").unwrap();

        let mut file = fs::File::open(&src).await.unwrap();
        let len = file.metadata().await.unwrap().len();
        let from_reader = store.copy_from_file(len, &mut file).await.unwrap();
        let from_bytes = store.put(&Bytes::from_static(b"hello")).await.unwrap();
        assert_eq!(from_reader, from_bytes);
    }

    #[tokio::test]
    async fn copy_from_produces_the_same_digest_as_put() {
        let (_dir, store) = store();
        let content = Bytes::from_static(b"hello");
        let mut cursor = io::Cursor::new(&content);
        let d = store.copy_from(content.len() as u64, &mut cursor).await.unwrap();
        assert_eq!(d, digest(b"hello"));
        assert_eq!(store.get(&d).await.unwrap(), Some(content));
    }

    #[tokio::test]
    async fn path_points_at_the_stored_file() {
        let (dir, store) = store();
        let d = store.put(&Bytes::from_static(b"hello")).await.unwrap();

        let path = store.path(&d);
        assert_eq!(path, dir.path().join(d.hex(2)));
        assert_eq!(std::fs::read(path).unwrap(), b"hello".to_vec());
    }

    #[tokio::test]
    async fn put_stores_content_read_only() {
        use std::os::unix::fs::PermissionsExt;

        let (dir, store) = store();
        let d = store.put(&Bytes::from_static(b"hello")).await.unwrap();

        let path = dir.path().join(d.hex(2));
        let perms = std::fs::metadata(&path).unwrap().permissions();
        assert!(perms.readonly());
        assert_eq!(perms.mode() & 0o200, 0);
    }

    #[tokio::test]
    async fn get_returns_an_error_for_corrupted_content_instead_of_panicking() {
        use std::os::unix::fs::PermissionsExt;

        let (dir, store) = store();
        let d = store.put(&Example { value: 1 }).await.unwrap();

        // Overwrite the stored bytes on disk with something that isn't
        // valid CBOR at all, simulating on-disk corruption.
        let path = dir.path().join(d.hex(2));
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&path, perms).unwrap();
        std::fs::write(&path, b"not cbor").unwrap();

        let err = store.get::<Example>(&d).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
