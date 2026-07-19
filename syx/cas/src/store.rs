//! A content-addressed blob store.
use std::future::{
    self,
    Future,
};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use async_compression::tokio::bufread::ZstdDecoder;
use async_compression::tokio::write::ZstdEncoder;
use bytes::Bytes;
use moka::future::Cache;
use tokio::io::{
    AsyncRead,
    AsyncReadExt as _,
    AsyncSeek,
    AsyncSeekExt as _,
    AsyncWriteExt as _,
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
    pub fn open(root: impl Into<PathBuf>) -> io::Result<Self> {
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
    /// it's actually been stored yet.
    ///
    /// The file at this path holds zstd-compressed bytes, not the raw
    /// content `digest` was computed from -- a caller that wants to work
    /// with the raw bytes directly (e.g. mmap it) instead of loading it
    /// via `get` must decompress it first.
    pub fn path(&self, digest: &Digest) -> PathBuf {
        self.root.join(digest.hex(2))
    }

    /// Reads the content at `digest`, if present.
    pub async fn get<T: FromBytes>(&self, digest: &Digest) -> io::Result<Option<T>> {
        let compressed = match fs::read(self.path(digest)).await {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };

        let mut bytes = Vec::new();
        ZstdDecoder::new(io::Cursor::new(compressed))
            .read_to_end(&mut bytes)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let content = T::from_bytes(Bytes::from(bytes))
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
        let file = fs::File::from_std(tmp.as_file().try_clone()?);
        let mut encoder = ZstdEncoder::new(file);

        let digest = {
            let mut h = Hasher::new();
            h.tee_read_from(len, r, &mut encoder).await?;
            h.digest()
        };
        encoder.shutdown().await?;

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
            let file = fs::File::from_std(tmp.as_file().try_clone()?);
            let mut encoder = ZstdEncoder::new(file);
            tokio::io::copy(r, &mut encoder).await?;
            encoder.shutdown().await?;
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
            let file = fs::File::from_std(tmp.as_file().try_clone()?);
            let mut encoder = ZstdEncoder::new(file);
            encoder.write_all(bytes.as_ref()).await?;
            encoder.shutdown().await?;
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
