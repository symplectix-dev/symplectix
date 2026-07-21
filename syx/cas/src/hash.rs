//! The digest primitive everything else in `cas` is addressed by.

use std::{
    fmt,
    io,
};

use bytes::Bytes;
use sha2::Digest as _;
use tokio::io::{
    AsyncRead,
    AsyncReadExt as _,
    AsyncWrite,
    AsyncWriteExt as _,
};

#[cfg(test)]
#[path = "hash_test.rs"]
mod tests;

/// Digest of `value`'s canonical byte encoding.
pub fn digest<T: ToBytes>(value: &T) -> Result<Digest, T::Error> {
    let bytes = value.to_bytes()?;
    let mut h = Hasher::new();
    h.part(bytes);
    Ok(h.digest())
}

/// This value's canonical byte encoding.
///
/// Implementations must keep this consistent with `FromBytes`:
/// `T::from_bytes(&x.to_bytes())` must equal `Ok(x)`, and encoding the
/// same logical value twice (in any order it was built) must always
/// produce identical bytes.
pub trait ToBytes {
    /// Why encoding can fail.
    type Error: fmt::Debug;

    /// Encode `self` into its canonical byte form.
    fn to_bytes(&self) -> Result<Bytes, Self::Error>;
}

/// The inverse of `ToBytes`.
/// Build a value back from its own byte encoding.
pub trait FromBytes: Sized {
    /// Why decoding can fail.
    type Error: fmt::Debug;

    /// Decode `bytes` back into `Self`.
    fn from_bytes(bytes: Bytes) -> Result<Self, Self::Error>;
}

impl ToBytes for Bytes {
    type Error = std::convert::Infallible;
    fn to_bytes(&self) -> Result<Bytes, Self::Error> {
        Ok(self.clone())
    }
}

impl FromBytes for Bytes {
    type Error = std::convert::Infallible;
    fn from_bytes(bytes: Bytes) -> Result<Self, Self::Error> {
        Ok(bytes)
    }
}

/// A digest's raw bytes.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Digest(#[serde(with = "serde_bytes")] [u8; 32]);

impl Digest {
    /// Wrap already-computed digest bytes.
    pub fn new(bytes: [u8; 32]) -> Self {
        Digest(bytes)
    }
}

impl AsRef<[u8]> for Digest {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::LowerHex for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

/// Read size for streaming a reader's bytes into a digest,
/// so a large part is never buffered whole in memory.
const BUF_SIZE: usize = 1 << 16;

/// Builds a length-prefixed `Digest` over an ordered sequence of parts.
///
/// This framing is self-delimiting, so no two distinct sequences
/// of parts produce the same digest. For example, `part(b"a").part(b"b")`
/// cannot collide with `part(b"ab")`.
pub struct Hasher {
    hasher: sha2::Sha256,
}

impl Default for Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher {
    /// A fresh `Hasher` with no parts folded in yet.
    pub fn new() -> Self {
        Hasher { hasher: sha2::Sha256::new() }
    }

    /// Fold one more part into the digest.
    pub fn part(&mut self, part: impl AsRef<[u8]>) -> &mut Self {
        let bytes = part.as_ref();
        self.hasher.update((bytes.len() as u64).to_be_bytes());
        self.hasher.update(bytes);
        self
    }

    /// Fold each part into the digest, in order.
    pub fn parts<I, T>(&mut self, parts: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: AsRef<[u8]>,
    {
        for part in parts {
            self.part(part);
        }
        self
    }

    /// Fold a part of known `len` bytes, read from `r`, into the digest.
    ///
    /// `len` is what says where this blob ends: a source that can outlive
    /// a single blob (a persistent socket, a multiplexed stream),
    /// EOF only marks the end of the whole connection, not of this blob.
    ///
    /// `r` is trusted to have at least `len` bytes available; an
    /// `AsyncRead` cannot report its own length up front without being
    /// fully consumed, so the caller must already know it. Returns an
    /// error if `r` runs out before `len` bytes are read. Reads exactly
    /// `len` bytes and no more, so any bytes in `r` beyond that are left
    /// untouched -- neither read nor checked.
    pub async fn read_from(
        &mut self,
        len: u64,
        r: impl AsyncRead + Unpin,
    ) -> io::Result<&mut Self> {
        self.tee_read_from(len, r, tokio::io::sink()).await
    }

    /// Like `read_from`, but also forwards each chunk to `w` as it's
    /// read, so a source can be hashed and copied to `w` in a single
    /// pass without materializing it whole in memory.
    pub async fn tee_read_from(
        &mut self,
        len: u64,
        mut r: impl AsyncRead + Unpin,
        mut w: impl AsyncWrite + Unpin,
    ) -> io::Result<&mut Self> {
        self.hasher.update(len.to_be_bytes());

        let mut remaining = len;
        let mut buf = [0u8; BUF_SIZE];
        while remaining > 0 {
            let want = remaining.min(BUF_SIZE as u64) as usize;
            let n = r.read(&mut buf[..want]).await?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!("reader ended {remaining} bytes short of the declared length {len}"),
                ));
            }
            self.hasher.update(&buf[..n]);
            w.write_all(&buf[..n]).await?;
            remaining -= n as u64;
        }
        Ok(self)
    }

    /// Finalize and return the digest's bytes.
    pub fn digest(self) -> Digest {
        Digest(self.hasher.finalize().into())
    }
}
