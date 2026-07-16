//! A content-addressed blob store.
use std::io;
use std::path::PathBuf;

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

impl ToBytes for Vec<u8> {
    type Error = std::convert::Infallible;

    fn to_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        // TODO: Remove clone.
        Ok(self.clone())
    }
}

impl FromBytes for Vec<u8> {
    type Error = std::convert::Infallible;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Ok(bytes.to_vec())
    }
}

/// A content-addressed store mapping `Digest` keys to blob content: an
/// in-memory cache in front of files under `root`, sharded the same way
/// as `Digest::hex`.
pub struct Store {
    root:  PathBuf,
    /// digest -> content length in bytes, not the content itself.
    cache: Cache<Digest, u64>,
}

impl Store {
    /// Open a store rooted at `root` (created lazily on first `put`),
    /// with an in-memory cache holding up to `max_capacity` entries.
    pub fn new(root: impl Into<PathBuf>, max_capacity: u64) -> Self {
        Store { root: root.into(), cache: Cache::new(max_capacity) }
    }

    /// The path content addressed by `digest` lives at (whether or not
    /// it's actually been stored yet), for a caller that wants to work
    /// with the file directly (e.g. mmap it) instead of loading it via
    /// `get`.
    pub fn path(&self, digest: &Digest) -> PathBuf {
        self.root.join(digest.hex(2))
    }

    /// Whether content is already stored at `digest`, in the cache or on
    /// disk.
    pub async fn exists(&self, digest: &Digest) -> io::Result<bool> {
        if self.cache.contains_key(digest) {
            return Ok(true);
        }
        match fs::metadata(self.path(digest)).await {
            Ok(meta) => {
                self.cache.insert(*digest, meta.len()).await;
                Ok(true)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// The content at `digest`, if present. Always reads from disk (the
    /// cache holds sizes, not content), and fills the cache with the
    /// content's size.
    pub async fn get<T: FromBytes>(&self, digest: &Digest) -> io::Result<Option<T>> {
        let bytes = match fs::read(self.path(digest)).await {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };

        let content = T::from_bytes(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e:?}")))?;

        self.cache.insert(*digest, bytes.len() as u64).await;
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

        if self.exists(&digest).await? {
            return Ok(digest);
        }

        let tmp = self.new_temp_file().await?;
        fs::write(tmp.path(), &bytes).await?;
        self.persist(tmp, digest, bytes.len() as u64).await?;
        Ok(digest)
    }

    /// Store the content read from a seekable `r` of `len` bytes.
    ///
    /// Reads `r` once to compute the digest; only if that digest isn't already
    /// stored does it seek back to the start and copy `r` in.
    pub async fn copy_from<R>(&self, len: u64, r: &mut R) -> io::Result<Digest>
    where
        R: AsyncRead + AsyncSeek + Unpin,
    {
        let digest = {
            let mut h = Hasher::new();
            h.read_from(len, &mut *r).await?;
            h.digest()
        };

        if self.exists(&digest).await? {
            return Ok(digest);
        }

        r.rewind().await?;
        let tmp = self.new_temp_file().await?;
        let mut file = fs::File::create(tmp.path()).await?;
        tokio::io::copy(r, &mut file).await?;
        self.persist(tmp, digest, len).await?;
        Ok(digest)
    }

    /// Store the content read from a one-shot `r` of `len` bytes
    /// that can't be rewound (e.g. a socket), addressed by its own digest.
    ///
    /// Hashes and writes `r` in a single pass, so unlike `copy_from`,
    /// an already-stored duplicate still costs one temp-file write.
    pub async fn copy_from_stream<R>(&self, len: u64, r: &mut R) -> io::Result<Digest>
    where
        R: AsyncRead + Unpin,
    {
        let tmp = self.new_temp_file().await?;
        let mut file = fs::File::create(tmp.path()).await?;

        let digest = {
            let mut h = Hasher::new();
            h.tee_read_from(len, r, &mut file).await?;
            h.digest()
        };

        if self.exists(&digest).await? {
            return Ok(digest);
        }

        self.persist(tmp, digest, len).await?;
        Ok(digest)
    }

    /// A fresh, empty temp file under `root`, on the same filesystem as
    /// every digest-addressed path this store persists to, so a later
    /// rename into place is atomic.
    async fn new_temp_file(&self) -> io::Result<tempfile::NamedTempFile> {
        let dir = self.root.join("tmp");
        fs::create_dir_all(&dir).await?;
        task::spawn_blocking(move || tempfile::NamedTempFile::new_in(dir))
            .await
            .expect("creating the temp file should not panic")
    }

    /// Make `tmp` read-only and rename it into place at `digest`'s path,
    /// then record its size in the cache.
    ///
    /// This is best-effort, not a concurrency-safe write-once: two
    /// writers racing on the same digest may both persist and the last
    /// one wins. That is harmless here because the path is a
    /// deterministic function of `digest`, so whichever write wins holds
    /// the intended content.
    async fn persist(
        &self,
        tmp: tempfile::NamedTempFile,
        digest: Digest,
        len: u64,
    ) -> io::Result<()> {
        let dst = self.path(&digest);
        let dir =
            dst.parent().expect("path is always root/xx/xx/rest, so it has a parent").to_owned();
        fs::create_dir_all(&dir).await?;

        task::spawn_blocking(move || -> io::Result<()> {
            // Read-only: a digest is only valid as long as the bytes it
            // was computed from never change, so CAS entries must not be
            // mutable once written. On Unix, deleting a file only needs
            // write permission on its directory, so this doesn't block
            // cleanup. But I'm not sure abount Windows. Removing a
            // Store's root there may need extra handling.
            let mut perms = tmp.as_file().metadata()?.permissions();
            perms.set_readonly(true);
            tmp.as_file().set_permissions(perms)?;
            tmp.persist(dst)?;
            Ok(())
        })
        .await
        .expect("persisting the temp file should not panic")?;

        self.cache.insert(digest, len).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Storable;

    fn digest(bytes: &[u8]) -> Digest {
        let mut h = Hasher::new();
        h.part(bytes);
        h.digest()
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Example {
        value: u32,
    }

    impl Storable for Example {}

    fn store() -> (testing::TempDir, Store) {
        let dir = testing::tempdir();
        let store = Store::new(dir.path(), 100);
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
        assert_eq!(store.get::<Vec<u8>>(&digest(b"missing")).await.unwrap(), None);
    }

    #[tokio::test]
    async fn put_then_get() {
        let (_dir, store) = store();
        let d = store.put(&b"hello".to_vec()).await.unwrap();
        assert!(store.exists(&d).await.unwrap());
        assert_eq!(store.get(&d).await.unwrap(), Some(b"hello".to_vec()));
    }

    #[tokio::test]
    async fn put_returns_the_content_digest() {
        let (_dir, store) = store();
        let d = store.put(&b"hello".to_vec()).await.unwrap();
        assert_eq!(d, digest(b"hello"));
    }

    #[tokio::test]
    async fn put_skips_writing_when_content_already_exists() {
        use std::os::unix::fs::PermissionsExt;

        let (_dir, store) = store();
        let d = store.put(&b"hello".to_vec()).await.unwrap();

        // Make the destination's parent directory read-only: if the
        // second put below actually tried to write (create a temp file
        // in it), that would now fail with a permission error.
        let path = store.path(&d);
        let dir = path.parent().unwrap();
        let permissions = std::fs::metadata(dir).unwrap().permissions();
        let mut readonly = permissions.clone();
        readonly.set_mode(0o500);
        std::fs::set_permissions(dir, readonly).unwrap();

        let d2 = store.put(&b"hello".to_vec()).await.unwrap();
        assert_eq!(d, d2);

        // Restore write permission so the tempdir can clean itself up.
        std::fs::set_permissions(dir, permissions).unwrap();
    }

    #[tokio::test]
    async fn put_persists_to_disk_not_just_the_cache() {
        // A fresh Store instance over the same root sees content a prior
        // instance wrote: proof it actually landed on disk, not only in
        // the (per-instance) in-memory cache.
        let dir = testing::tempdir();

        let writer = Store::new(dir.path(), 100);
        let d = writer.put(&b"hello".to_vec()).await.unwrap();

        let reader = Store::new(dir.path(), 100);
        assert!(reader.exists(&d).await.unwrap());
        assert_eq!(reader.get(&d).await.unwrap(), Some(b"hello".to_vec()));
    }

    #[tokio::test]
    async fn get_reads_through_to_disk_on_cache_miss() {
        // Content written by one Store is visible via a second Store's
        // get(), which must fall back to disk since its own cache never
        // saw this digest.
        let dir = testing::tempdir();

        let writer = Store::new(dir.path(), 100);
        let d = writer.put(&b"hello".to_vec()).await.unwrap();

        let reader = Store::new(dir.path(), 100);
        assert_eq!(reader.get(&d).await.unwrap(), Some(b"hello".to_vec()));
    }

    #[tokio::test]
    async fn content_is_sharded_under_root() {
        let (dir, store) = store();
        let d = store.put(&b"hello".to_vec()).await.unwrap();

        let path = dir.path().join(d.hex(2));
        assert_eq!(std::fs::read(path).unwrap(), b"hello".to_vec());
    }

    #[tokio::test]
    async fn copy_from_accepts_a_file_and_streams_it_in() {
        // A file already on disk (not just in-memory bytes) can be
        // ingested via copy_from, streamed in without requiring the
        // caller to load it into memory first.
        let (_dir, store) = store();
        let src_dir = testing::tempdir();
        let src = src_dir.path().join("blob");
        std::fs::write(&src, b"hello").unwrap();

        let mut file = fs::File::open(&src).await.unwrap();
        let len = file.metadata().await.unwrap().len();
        let d = store.copy_from(len, &mut file).await.unwrap();
        assert_eq!(d, digest(b"hello"));
        assert_eq!(store.get(&d).await.unwrap(), Some(b"hello".to_vec()));
    }

    #[tokio::test]
    async fn copy_from_produces_the_same_digest_as_put() {
        let (_dir, store) = store();
        let src_dir = testing::tempdir();
        let src = src_dir.path().join("blob");
        std::fs::write(&src, b"hello").unwrap();

        let mut file = fs::File::open(&src).await.unwrap();
        let len = file.metadata().await.unwrap().len();
        let from_reader = store.copy_from(len, &mut file).await.unwrap();
        let from_bytes = store.put(&b"hello".to_vec()).await.unwrap();
        assert_eq!(from_reader, from_bytes);
    }

    #[tokio::test]
    async fn copy_from_stream_produces_the_same_digest_as_put() {
        let (_dir, store) = store();
        let content = b"hello".to_vec();
        let mut cursor = io::Cursor::new(&content);
        let d = store.copy_from_stream(content.len() as u64, &mut cursor).await.unwrap();
        assert_eq!(d, digest(b"hello"));
        assert_eq!(store.get(&d).await.unwrap(), Some(content));
    }

    #[tokio::test]
    async fn path_points_at_the_stored_file() {
        let (dir, store) = store();
        let d = store.put(&b"hello".to_vec()).await.unwrap();

        let path = store.path(&d);
        assert_eq!(path, dir.path().join(d.hex(2)));
        assert_eq!(std::fs::read(path).unwrap(), b"hello".to_vec());
    }

    #[tokio::test]
    async fn put_stores_content_read_only() {
        use std::os::unix::fs::PermissionsExt;

        let (dir, store) = store();
        let d = store.put(&b"hello".to_vec()).await.unwrap();

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
