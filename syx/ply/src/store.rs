//! Store: a content-addressed blob store.
//!
//! An in-memory cache in front of a local-filesystem directory (no S3
//! support yet). `put` writes through to disk before returning, so
//! content survives a cache eviction or a process restart. The cache
//! holds only digest -> size, not blob content, so its footprint does
//! not grow with blob size; it exists purely to make repeated `exists`
//! checks fast without a `stat` syscall.

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
    Digest,
    Hasher,
};

/// Content accepted by `Store::put`: either in-memory bytes or a file
/// already on local disk. A `PathBuf` is hashed and written by
/// streaming its bytes, so a file already on disk never gets buffered
/// whole in memory just to be written straight back out.
pub trait Content: Send + 'static {
    /// This content's digest.
    fn digest(&self) -> impl Future<Output = io::Result<Digest>> + Send;

    /// Write this content into a fresh file at `dst`, returning its
    /// length in bytes.
    fn write_to(&self, dst: &Path) -> impl Future<Output = io::Result<u64>> + Send;
}

impl Content for Vec<u8> {
    async fn digest(&self) -> io::Result<Digest> {
        let mut h = Hasher::new();
        h.part(self);
        Ok(h.digest())
    }

    async fn write_to(&self, dst: &Path) -> io::Result<u64> {
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

    async fn write_to(&self, dst: &Path) -> io::Result<u64> {
        fs::copy(self, dst).await
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

    /// The blob at `digest`, if present. Always reads from disk (the
    /// cache holds sizes, not content), and fills the cache with the
    /// content's size.
    pub async fn get(&self, digest: &Digest) -> io::Result<Option<Vec<u8>>> {
        match fs::read(self.path(digest)).await {
            Ok(content) => {
                self.cache.insert(*digest, content.len() as u64).await;
                Ok(Some(content))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Store `content` (in-memory bytes or an existing local file),
    /// addressed by its own digest, and return that digest.
    ///
    /// If the address already exists, the write is skipped, but this is
    /// best-effort, not a concurrency-safe write-once: two writers racing
    /// on the same address may both write and the last one wins. That is
    /// harmless here because the address is a deterministic function of
    /// `content`, so whichever write wins holds the intended content.
    ///
    /// The write to disk is atomic: `content` lands in a temp file next
    /// to the destination, then gets renamed into place, so a concurrent
    /// reader never observes a partially-written file.
    pub async fn put(&self, content: impl Content) -> io::Result<Digest> {
        let digest = content.digest().await?;

        if self.exists(&digest).await? {
            return Ok(digest);
        }

        let dst = self.path(&digest);
        let dir =
            dst.parent().expect("path is always root/xx/xx/rest, so it has a parent").to_owned();
        fs::create_dir_all(&dir).await?;

        // NamedTempFile reserves a uniquely-named file in `dir` so the
        // final rename is atomic and on the same filesystem; creating it
        // is sync (std::fs), so it runs on a blocking thread.
        let tmp = task::spawn_blocking(move || tempfile::NamedTempFile::new_in(dir))
            .await
            .expect("blocking task should not panic")?;

        let len = content.write_to(tmp.path()).await?;

        task::spawn_blocking(move || -> io::Result<()> {
            tmp.persist(dst)?;
            Ok(())
        })
        .await
        .expect("blocking task should not panic")?;

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

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path(), 100);
        (dir, store)
    }

    #[tokio::test]
    async fn missing_digest_does_not_exist() {
        let (_dir, store) = store();
        assert!(!store.exists(&digest(b"hello")).await.unwrap());
    }

    #[tokio::test]
    async fn put_then_get_round_trips() {
        let (_dir, store) = store();
        let d = store.put(b"hello".to_vec()).await.unwrap();
        assert!(store.exists(&d).await.unwrap());
        assert_eq!(store.get(&d).await.unwrap(), Some(b"hello".to_vec()));
    }

    #[tokio::test]
    async fn put_returns_the_content_digest() {
        let (_dir, store) = store();
        let d = store.put(b"hello".to_vec()).await.unwrap();
        assert_eq!(d, digest(b"hello"));
    }

    #[tokio::test]
    async fn get_missing_digest_is_none() {
        let (_dir, store) = store();
        assert_eq!(store.get(&digest(b"missing")).await.unwrap(), None);
    }

    #[tokio::test]
    async fn put_is_idempotent_for_the_same_content() {
        // A second put of the same content is a no-op (put skips the
        // write if the address already exists), and still returns the
        // same digest.
        let (_dir, store) = store();
        let a = store.put(b"hello".to_vec()).await.unwrap();
        let b = store.put(b"hello".to_vec()).await.unwrap();
        assert_eq!(a, b);
        assert_eq!(store.get(&a).await.unwrap(), Some(b"hello".to_vec()));
    }

    #[tokio::test]
    async fn put_persists_to_disk_not_just_the_cache() {
        // A fresh Store instance over the same root sees content a prior
        // instance wrote: proof it actually landed on disk, not only in
        // the (per-instance) in-memory cache.
        let dir = tempfile::tempdir().unwrap();

        let writer = Store::new(dir.path(), 100);
        let d = writer.put(b"hello".to_vec()).await.unwrap();

        let reader = Store::new(dir.path(), 100);
        assert!(reader.exists(&d).await.unwrap());
        assert_eq!(reader.get(&d).await.unwrap(), Some(b"hello".to_vec()));
    }

    #[tokio::test]
    async fn get_reads_through_to_disk_on_cache_miss() {
        // Content written by one Store is visible via a second Store's
        // get(), which must fall back to disk since its own cache never
        // saw this digest.
        let dir = tempfile::tempdir().unwrap();

        let writer = Store::new(dir.path(), 100);
        let d = writer.put(b"hello".to_vec()).await.unwrap();

        let reader = Store::new(dir.path(), 100);
        assert_eq!(reader.get(&d).await.unwrap(), Some(b"hello".to_vec()));
    }

    #[tokio::test]
    async fn content_is_sharded_under_root() {
        let (dir, store) = store();
        let d = store.put(b"hello".to_vec()).await.unwrap();

        let path = dir.path().join(d.hex(2));
        assert_eq!(std::fs::read(path).unwrap(), b"hello".to_vec());
    }

    #[tokio::test]
    async fn put_accepts_a_path_and_streams_it_in() {
        // A file already on disk (not just in-memory bytes) can be
        // put()'d directly; the store reads it in via Content::digest /
        // write_into rather than requiring the caller to load it first.
        let (_dir, store) = store();
        let src_dir = tempfile::tempdir().unwrap();
        let src = src_dir.path().join("blob");
        std::fs::write(&src, b"hello").unwrap();

        let d = store.put(src).await.unwrap();
        assert_eq!(d, digest(b"hello"));
        assert_eq!(store.get(&d).await.unwrap(), Some(b"hello".to_vec()));
    }

    #[tokio::test]
    async fn path_and_bytes_content_produce_the_same_digest() {
        let (_dir, store) = store();
        let src_dir = tempfile::tempdir().unwrap();
        let src = src_dir.path().join("blob");
        std::fs::write(&src, b"hello").unwrap();

        let from_path = store.put(src).await.unwrap();
        let from_bytes = store.put(b"hello".to_vec()).await.unwrap();
        assert_eq!(from_path, from_bytes);
    }

    #[tokio::test]
    async fn action_and_its_command_and_input_resolve_from_store() {
        use crate::action::{
            Action,
            Command,
        };
        use crate::blob::{
            Collection,
            Node,
        };

        let (_dir, store) = store();

        // The input: a Tree with one file entry, itself resolvable from
        // the store.
        let file_digest = store.put(b"print('hi')".to_vec()).await.unwrap();
        let input = Collection::tree([("main.py".to_string(), Node::Blob(file_digest))], []);
        let input_bytes = cbor2::to_canonical_vec(&input).unwrap();
        let input_digest = store.put(input_bytes).await.unwrap();
        assert_eq!(input_digest, input.digest());

        // The command to run against that input.
        let mut command = Command::new("python3");
        command.arg("main.py");
        let command_bytes = cbor2::to_canonical_vec(&command).unwrap();
        let command_digest = store.put(command_bytes).await.unwrap();
        assert_eq!(command_digest, command.digest());

        // The action tying command and input together.
        let action = Action::new(command_digest, input_digest);
        let action_bytes = cbor2::to_canonical_vec(&action).unwrap();
        let action_digest = store.put(action_bytes).await.unwrap();
        assert_eq!(action_digest, action.digest());

        // Read the whole graph back out of the store using only the
        // action's digest -- the resolution an executor would do before
        // actually running it.
        let stored_action_bytes = store.get(&action_digest).await.unwrap().unwrap();
        let resolved_action = Action::try_from(stored_action_bytes.as_slice()).unwrap();
        assert_eq!(resolved_action, action);

        let stored_command_bytes = store.get(&resolved_action.command()).await.unwrap().unwrap();
        let resolved_command = Command::try_from(stored_command_bytes.as_slice()).unwrap();
        assert_eq!(resolved_command, command);

        let stored_input_bytes = store.get(&resolved_action.input()).await.unwrap().unwrap();
        let resolved_input = Collection::try_from(stored_input_bytes.as_slice()).unwrap();
        assert_eq!(resolved_input.digest(), input.digest());

        // The file the input tree references is itself resolvable.
        assert_eq!(store.get(&file_digest).await.unwrap(), Some(b"print('hi')".to_vec()));
    }
}
