use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use content_addressing::{
    Chunking,
    Codec,
};
use logger::Logger;
use object_store::ObjectStore;
use object_store::local::LocalFileSystem;
use tokio::fs;

use crate::{
    DEFAULT_CAS_PREFIX,
    DEFAULT_DB_PREFIX,
    DEFAULT_FLUSH_THRESHOLD,
    DEFAULT_MAX_LOGGER_DURATION,
    DEFAULT_MAX_PENDING_SEGMENTS,
    Flushing,
    KeyDir,
    PackTarget,
    Storage,
    spawn_flush_loop,
};

/// Builds a [`Storage`].
///
/// `dir` is the only required argument: the local directory
/// not-yet-packed blobs are held in. `db` and `blobs` default to a local
/// `object_store` under it, so a store opens with no external setup;
/// override `db_backend`/`blobs` to put them on S3 or another backend.
pub struct Builder {
    dir: PathBuf,
    max_logger_duration: Option<Duration>,
    max_pending_segments: Option<u16>,
    db_prefix: Option<String>,
    db_backend: Option<Arc<dyn ObjectStore>>,
    blobs_backend: Option<Arc<dyn ObjectStore>>,
    flush_threshold: Option<u64>,
    cas_prefix: Option<String>,
    chunking: Option<Chunking>,
    codec: Option<Codec>,
}

impl Builder {
    fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            max_logger_duration: None,
            max_pending_segments: None,
            db_prefix: None,
            db_backend: None,
            blobs_backend: None,
            flush_threshold: None,
            cas_prefix: None,
            chunking: None,
            codec: None,
        }
    }

    /// How long `Logger` lets a segment stay active before rotating
    /// it out on its own, regardless of `flush_threshold`.
    pub fn max_logger_duration(mut self, max_logger_duration: Duration) -> Self {
        self.max_logger_duration = Some(max_logger_duration);
        self
    }

    /// How many pending (rotated, not yet packed) segments `Logger`
    /// lets accumulate before refusing further writes.
    pub fn max_pending_segments(mut self, max_pending_segments: u16) -> Self {
        self.max_pending_segments = Some(max_pending_segments);
        self
    }

    /// The key prefix `db` itself is opened under, within `db_backend`.
    /// Only needed when `db_backend` is shared with something else that
    /// also needs a prefix of its own.
    pub fn db_prefix(mut self, db_prefix: impl Into<String>) -> Self {
        self.db_prefix = Some(db_prefix.into());
        self
    }

    /// Where `db` (the pointer store) lives. Defaults to a local
    /// `object_store` under `dir` when not set.
    pub fn db_backend(mut self, db_backend: Arc<dyn ObjectStore>) -> Self {
        self.db_backend = Some(db_backend);
        self
    }

    /// Where packed blob objects live. Defaults to `db_backend` (the
    /// resolved one, whether explicit or defaulted) when not set.
    pub fn blobs(mut self, blobs: Arc<dyn ObjectStore>) -> Self {
        self.blobs_backend = Some(blobs);
        self
    }

    /// How many bytes `Logger` stages before rotating a segment out
    /// on its own and making it worth consolidating into a pack.
    pub fn flush_threshold(mut self, flush_threshold: u64) -> Self {
        self.flush_threshold = Some(flush_threshold);
        self
    }

    /// The key prefix blobs are packed under, within `blobs`. Unrelated
    /// to `db_prefix`.
    pub fn cas_prefix(mut self, cas_prefix: impl Into<String>) -> Self {
        self.cas_prefix = Some(cas_prefix.into());
        self
    }

    // TODO: `chunking` and `codec` can be overridden independently, but
    // `Codec::SNIFF_LEN` should stay below `Chunking::MIN_SIZE`. Nothing
    // breaks if it happens, but `Codec::encode` compresses a chunk twice
    // instead of once.

    /// Overrides chunking behavior.
    pub fn chunking(mut self, chunking: Chunking) -> Self {
        self.chunking = Some(chunking);
        self
    }

    /// Overrides encoding/decoding behavior.
    pub fn codec(mut self, codec: Codec) -> Self {
        self.codec = Some(codec);
        self
    }

    /// Opens `db` and the log, and spawns the task that packs whatever
    /// the log rotates out.
    pub async fn build(self) -> io::Result<Storage> {
        let Self {
            dir,
            max_logger_duration,
            max_pending_segments,
            db_prefix,
            db_backend,
            blobs_backend,
            flush_threshold,
            cas_prefix,
            chunking,
            codec,
        } = self;

        let db_backend = match db_backend {
            Some(backend) => backend,
            None => {
                let dir = dir.join("db");
                fs::create_dir_all(&dir).await?;
                let local = LocalFileSystem::new_with_prefix(dir).map_err(io::Error::other)?;
                Arc::new(local) as Arc<dyn ObjectStore>
            }
        };
        let db_prefix = db_prefix.unwrap_or_else(|| DEFAULT_DB_PREFIX.to_string());
        let db = slatedb::Db::builder(db_prefix, db_backend.clone())
            .build()
            .await
            .map_err(io::Error::other)?;

        let codec = codec.unwrap_or_default();
        let max_pending_segments = max_pending_segments.unwrap_or(DEFAULT_MAX_PENDING_SEGMENTS);
        let flush_threshold = flush_threshold.unwrap_or(DEFAULT_FLUSH_THRESHOLD);
        let max_logger_duration = max_logger_duration.unwrap_or(DEFAULT_MAX_LOGGER_DURATION);
        // `Logger::open` claims a whole directory, treating every
        // matching entry in it as one of its own segments, so it gets a
        // subdirectory rather than `dir` itself, which `db` and `blobs`
        // also default into. One threshold drives both rotation and
        // packing: the size worth consolidating into a pack is the size
        // worth rotating out of the active slot.
        let (logger, replayed) = Logger::open(
            dir.join("logger"),
            max_pending_segments,
            flush_threshold,
            Some(max_logger_duration),
        )
        .await?;
        let rotated = logger.rotated();
        let logger = Arc::new(logger);
        let staged = Arc::new(KeyDir::rebuild(replayed, codec).await);

        let blobs = blobs_backend.unwrap_or_else(|| db_backend.clone());
        let chunking = chunking.unwrap_or_default();
        let cas_prefix: Arc<str> =
            Arc::from(cas_prefix.unwrap_or_else(|| DEFAULT_CAS_PREFIX.to_string()));

        let flushing = Flushing::new();
        // Reacts to rotation events, so a segment still gets packed when
        // no further write follows it.
        spawn_flush_loop(
            Arc::downgrade(&logger),
            rotated,
            PackTarget {
                db:         db.clone(),
                blobs:      Arc::clone(&blobs),
                cas_prefix: Arc::clone(&cas_prefix),
                staged:     Arc::clone(&staged),
                flushing:   flushing.clone(),
            },
        );
        Ok(Storage::new(logger, staged, db, blobs, flushing, cas_prefix, chunking, codec))
    }
}

impl Storage {
    /// Starts building a `Storage`. See [`Builder`]'s own doc for
    /// what's required vs. defaulted.
    pub fn builder(dir: impl Into<PathBuf>) -> Builder {
        Builder::new(dir)
    }
}
