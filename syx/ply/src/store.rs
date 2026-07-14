//! A content-addressed blob store.
use std::future::Future;
use std::io;
use std::path::{
    Path,
    PathBuf,
};

use moka::future::Cache;
use tokio::{
    fs,
    task,
};

use crate::hash::{
    self,
    Digest,
    Hasher,
};

/// Content managed by `Store`.
pub trait Content: 'static + Send {
    /// This content's digest.
    fn digest(&self) -> impl Future<Output = io::Result<Digest>> + Send;

    /// Read the content back from `src`.
    fn read_from<P>(src: &P) -> impl Future<Output = io::Result<Self>> + Send
    where
        Self: Sized,
        P: ?Sized + Sync + AsRef<Path>;

    /// Write this content into a fresh file at `dst`,
    /// returning its length in bytes.
    fn write_to<P>(&self, dst: &P) -> impl Future<Output = io::Result<u64>> + Send
    where
        P: ?Sized + Sync + AsRef<Path>;
}

impl Content for Vec<u8> {
    async fn digest(&self) -> io::Result<Digest> {
        let mut h = Hasher::new();
        h.part(self);
        Ok(h.digest())
    }

    async fn read_from<P>(src: &P) -> io::Result<Self>
    where
        P: ?Sized + Sync + AsRef<Path>,
    {
        fs::read(src).await
    }

    async fn write_to<P>(&self, dst: &P) -> io::Result<u64>
    where
        P: ?Sized + Sync + AsRef<Path>,
    {
        fs::write(dst, self).await?;
        Ok(self.len() as u64)
    }
}

impl Content for PathBuf {
    async fn digest(&self) -> io::Result<Digest> {
        let mut file = fs::File::open(self).await?;
        let len = file.metadata().await?.len();

        let mut h = Hasher::new();
        h.async_reader(len, &mut file).await?;
        Ok(h.digest())
    }

    /// A `PathBuf`'s "content" is a file elsewhere; reading it back just
    /// means pointing at where the store already put it, not loading it.
    async fn read_from<P>(src: &P) -> io::Result<Self>
    where
        P: AsRef<Path> + ?Sized + Sync,
    {
        Ok(src.as_ref().to_path_buf())
    }

    async fn write_to<P>(&self, dst: &P) -> io::Result<u64>
    where
        P: AsRef<Path> + ?Sized + Sync,
    {
        fs::copy(self, dst).await
    }
}

/// Marker for types that are stored as their own canonical CBOR encoding,
/// as opposed to `Vec<u8>`/`PathBuf`, which store already-raw bytes/files.
pub(crate) trait Storable:
    serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static
{
}

impl<T: Storable> Content for T {
    async fn digest(&self) -> io::Result<Digest> {
        Ok(hash::digest_of(self))
    }

    async fn read_from<P>(src: &P) -> io::Result<Self>
    where
        P: ?Sized + Sync + AsRef<Path>,
    {
        let bytes = fs::read(src).await?;
        cbor2::from_slice(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_to<P>(&self, dst: &P) -> io::Result<u64>
    where
        P: ?Sized + Sync + AsRef<Path>,
    {
        let bytes = cbor2::to_canonical_vec(self).expect("serializing to CBOR should not fail");
        fs::write(dst, &bytes).await?;
        Ok(bytes.len() as u64)
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

    fn path(&self, digest: &Digest) -> PathBuf {
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
    pub async fn get<T: Content>(&self, digest: &Digest) -> io::Result<Option<T>> {
        let path = self.path(digest);
        let content = match T::read_from(&path).await {
            Ok(content) => content,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };

        let len = fs::metadata(&path).await?.len();
        self.cache.insert(*digest, len).await;

        Ok(Some(content))
    }

    /// Store `content` (in-memory bytes or an existing local file),
    /// addressed by its own digest, and return that digest.
    ///
    /// The write to disk is atomic: `content` lands in a temp file next
    /// to the destination, then gets renamed into place, so a concurrent
    /// reader never observes a partially-written file.
    pub async fn put<T: Content>(&self, content: &T) -> io::Result<Digest> {
        let digest = content.digest().await?;

        // If the address already exists, the write is skipped.
        // This is best-effort, not a concurrency-safe write-once: two writers racing
        // on the same address may both write and the last one wins. That is harmless
        // here because the address is a deterministic function of `content`,
        // so whichever write wins holds the intended content.
        if self.exists(&digest).await? {
            return Ok(digest);
        }

        let dst = self.path(&digest);
        let dir =
            dst.parent().expect("path is always root/xx/xx/rest, so it has a parent").to_owned();
        fs::create_dir_all(&dir).await?;

        // About `expect(...)?`:
        // This `expect` unwraps Result<T, JoinError>, and the inner
        // io::Result is propagated normally via `?`.
        // Assuming NamedTempFile::new_in/persist does not panic.

        // NamedTempFile reserves a uniquely-named file in `dir` so the
        // final rename is on the same filesystem thus atomic.
        let tmp = task::spawn_blocking(move || tempfile::NamedTempFile::new_in(dir))
            .await
            .expect("creating the temp file should not panic")?;

        let len = content.write_to(tmp.path()).await?;

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

        self.cache.insert(digest, len).await;
        Ok(digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(bytes: &[u8]) -> Digest {
        let mut h = Hasher::new();
        h.part(bytes);
        h.digest()
    }

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
    async fn put_accepts_a_path_and_streams_it_in() {
        // A file already on disk (not just in-memory bytes) can be
        // put()'d directly; the store reads it in via Content::digest /
        // write_into rather than requiring the caller to load it first.
        let (_dir, store) = store();
        let src_dir = testing::tempdir();
        let src = src_dir.path().join("blob");
        std::fs::write(&src, b"hello").unwrap();

        let d = store.put(&src).await.unwrap();
        assert_eq!(d, digest(b"hello"));
        assert_eq!(store.get(&d).await.unwrap(), Some(b"hello".to_vec()));
    }

    #[tokio::test]
    async fn path_and_bytes_content_produce_the_same_digest() {
        let (_dir, store) = store();
        let src_dir = testing::tempdir();
        let src = src_dir.path().join("blob");
        std::fs::write(&src, b"hello").unwrap();

        let from_path = store.put(&src).await.unwrap();
        let from_bytes = store.put(&b"hello".to_vec()).await.unwrap();
        assert_eq!(from_path, from_bytes);
    }

    #[tokio::test]
    async fn get_as_path_points_at_the_stored_file_without_loading_it() {
        let (dir, store) = store();
        let d = store.put(&b"hello".to_vec()).await.unwrap();

        let path: PathBuf = store.get(&d).await.unwrap().unwrap();
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
}
