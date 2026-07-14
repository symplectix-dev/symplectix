//! cas: content-addressed storage.

mod hash;
mod store;

pub use hash::{
    digest,
    from_bytes,
    to_bytes,
    Digest,
    Error,
    Hasher,
};
pub use store::{
    Content,
    Storable,
    Store,
};
