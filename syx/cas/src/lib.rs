//! cas: content-addressed storage.

mod hash;
mod store;

pub use hash::{
    Digest,
    Hasher,
    Storable,
};
pub use store::{
    Content,
    Store,
};
