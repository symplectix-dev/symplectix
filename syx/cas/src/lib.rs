//! cas: content-addressed storage.

mod hash;
mod store;

pub use bytes::Bytes;
pub use hash::{
    Digest,
    FromBytes,
    Hasher,
    Storable,
    ToBytes,
    digest,
};
pub use store::Store;
