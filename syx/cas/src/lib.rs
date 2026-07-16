//! cas: content-addressed storage.

mod hash;
mod store;

pub use hash::{
    Digest,
    FromBytes,
    Hasher,
    Storable,
    ToBytes,
    digest,
};
pub use store::{
    Content,
    Store,
};
