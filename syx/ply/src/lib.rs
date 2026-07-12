//! ply: content addressing.

mod function;
mod hash;

pub use function::{
    Config,
    Function,
};
pub use hash::{
    Digest,
    Hasher,
};
