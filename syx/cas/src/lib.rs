//! cas: content-addressed storage.

mod hash;
mod store;

pub use bytes::Bytes;
pub use hash::{
    Digest,
    FromBytes,
    Hasher,
    ToBytes,
    digest,
};
pub use store::{
    Storage,
    copy_from,
    get,
    put,
    read_into,
};
