//! forgetter: a content-addressed, append-only log.
//!
//! `Logger` is the durable, content-agnostic append-only primitive (see
//! its own module doc). `Forgetter` layers content addressing on top of
//! it: chunking and encoding on the way in, packing not-yet-consolidated
//! segments into `object_store` packs.

use std::sync::Arc;

use object_store::ObjectStore;

mod builder;
mod logger;
mod storage;

pub use builder::Builder;
pub use logger::{
    FileId,
    Found,
    Locator,
    Logger,
    Replay,
    Segment,
    Slot,
};
pub use storage::Cas;

/// Content-addressed blob storage, built on `Logger`.
#[derive(Clone)]
pub struct Forgetter {
    // Durably holds not-yet-packed content until it's forgotten (packed
    // elsewhere). The only thing `Forgetter` can't default: everything
    // below can fall back to living under the same directory.
    logger: Arc<Logger>,
    // Maps a blob's digest to where `logger` is holding it; see
    // `storage::KeyDir`'s own doc for why this lives here and not
    // in `Logger` itself.
    staged: Arc<storage::KeyDir>,

    // `db`: maps a digest to its packed location.
    db: slatedb::Db,

    // Packed blob object storage, and when to consolidate `logger`'s
    // content into it.
    blobs:    Arc<dyn ObjectStore>,
    flushing: storage::Flushing,

    // Content addressing, applies uniformly regardless of backend.
    cas_prefix: Arc<str>,
    chunking:   content_addressing::Chunking,
    codec:      content_addressing::Codec,
}
