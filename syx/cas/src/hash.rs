//! The digest primitive everything else in `cas` is addressed by.

use std::fmt::Write as _;
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

/// Digest of `value`'s canonical byte encoding.
pub fn digest<T: ToBytes>(value: &T) -> Digest {
    let bytes = value
        .to_bytes()
        // Panicking here instead of returning a Result is safe for Vec<u8>/Bytes: Self::Error is
        // Infallible, so to_bytes() can't fail. For a type whose ToBytes goes through CBOR (e.g.
        // via cbor2::to_canonical_vec), the same holds in practice: that only fails on an I/O
        // error from the writer (impossible when writing into an in-memory Vec<u8>) or a value
        // CBOR can't represent, like NaN as a map key. `derive(Serialize)` always turns struct
        // fields into string keys, and `HashMap`/`BTreeMap` require `Eq + Hash`/`Ord` on their key
        // type, which `f32`/`f64` don't implement, so an ordinary derived struct/enum can't put a
        // float (let alone a NaN) in a map key position to begin with.
        .expect("serializing to bytes failed");
    let mut h = Hasher::new();
    h.part(bytes);
    h.digest()
}

/// This value's canonical byte encoding.
///
/// Implementations must keep this consistent with `FromBytes`:
/// `T::from_bytes(&x.to_bytes())` must equal `Ok(x)`, and encoding the
/// same logical value twice (in any order it was built) must always
/// produce identical bytes.
pub trait ToBytes {
    type Error: fmt::Debug;
    fn to_bytes(&self) -> Result<Bytes, Self::Error>;
}

/// The inverse of `ToBytes`.
/// Build a value back from its own byte encoding.
pub trait FromBytes: Sized {
    type Error: fmt::Debug;
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
    pub fn new(bytes: [u8; 32]) -> Self {
        Digest(bytes)
    }

    /// Format as a sharded hex string: `depth` leading two-character
    /// segments, then the rest, joined by `/`. For example, depth=3
    /// gives "ab/cd/ef/<remaining 58 hex chars>". depth=0 means no
    /// sharding. `depth` must be less than 32.
    pub fn hex(&self, depth: usize) -> String {
        assert!(depth < 32, "depth must be less than 32, got {depth}");
        let mut out = String::with_capacity(self.0.len() * 2 + depth);
        for (i, b) in self.0.iter().enumerate() {
            write!(out, "{b:02x}").expect("writing to a String never fails");
            if i < depth {
                out.push('/');
            }
        }
        out
    }
}

impl AsRef<[u8]> for Digest {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::UpperHex for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{b:02X}")?;
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// Digest of `parts`, combined in order: equal parts (in the same order)
    /// always give the same digest, different parts almost surely give
    /// different ones. Pass content alone to content-address it, or fold in
    /// metadata parts to address that too.
    fn digest<I, T>(parts: I) -> Digest
    where
        I: IntoIterator<Item = T>,
        T: AsRef<[u8]>,
    {
        let mut h = Hasher::new();
        h.parts(parts);
        h.digest()
    }

    #[test]
    fn digest_is_deterministic() {
        assert_eq!(digest([b"a".as_slice()]), digest([b"a".as_slice()]),);
    }

    #[test]
    fn digest_from_array_round_trips() {
        let want = digest([b"hello"]);
        let bytes: [u8; 32] = want.as_ref().try_into().unwrap();
        assert_eq!(Digest::new(bytes), want);
    }

    #[test]
    fn order_matters() {
        assert_ne!(
            digest([b"a".as_slice(), b"b".as_slice()]),
            digest([b"b".as_slice(), b"a".as_slice()]),
        );
    }

    #[test]
    fn framing_is_injective() {
        // Length-prefixing must stop `("a", "b")` from colliding with `("ab",)`.
        assert_ne!(digest([b"a".as_slice(), b"b".as_slice()]), digest([b"ab".as_slice()]));
    }

    #[test]
    fn builder_matches_free_function() {
        let mut h = Hasher::new();
        h.part(b"a").part(b"b");
        assert_eq!(h.digest(), digest([b"a".as_slice(), b"b".as_slice()]));
    }

    #[test]
    fn empty_parts_is_stable() {
        assert_eq!(digest(Vec::<&[u8]>::new()), digest(Vec::<&[u8]>::new()));
    }

    #[test]
    fn hex_depth_zero_is_plain_hex() {
        let d = digest([b"hello".as_slice()]);
        assert_eq!(d.hex(0), format!("{d:x}"));
    }

    #[test]
    fn hex_is_lowercase_64_chars() {
        let d = digest([b"hello".as_slice()]);
        let hex = d.hex(0);
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn hex_splits_leading_bytes() {
        let d = digest([b"hello".as_slice()]);
        let plain = format!("{d:x}");
        let want = format!("{}/{}/{}/{}", &plain[0..2], &plain[2..4], &plain[4..6], &plain[6..]);
        assert_eq!(d.hex(3), want);
    }

    #[test]
    #[should_panic]
    fn sharded_hex_rejects_depth_32_or_more() {
        digest([b"hello".as_slice()]).hex(32);
    }

    #[tokio::test]
    async fn async_reader_matches_in_memory_bytes() {
        let content = b"hello, reader".to_vec();
        let mut h = Hasher::new();
        h.read_from(content.len() as u64, io::Cursor::new(&content)).await.unwrap();
        assert_eq!(h.digest(), digest([content.as_slice()]));
    }

    #[tokio::test]
    async fn async_reader_is_chunked_not_buffered_at_once() {
        // Content larger than one BUF_SIZE read still hashes correctly,
        // proving the reader loops instead of assuming a single read call
        // drains everything.
        let content = vec![0x42u8; BUF_SIZE * 2 + 1];
        let mut h = Hasher::new();
        h.read_from(content.len() as u64, io::Cursor::new(&content)).await.unwrap();
        assert_eq!(h.digest(), digest([content.as_slice()]));
    }

    #[tokio::test]
    async fn async_reader_works_with_a_real_file() {
        // A real tokio::fs::File (not just an in-memory Cursor) implements
        // AsyncRead the same way; the caller supplies the length via `stat()`.
        let path = testing::rlocation("_main/.rustfmt.toml");
        let content = fs::read(&path).unwrap();
        let file = tokio::fs::File::open(&path).await.unwrap();
        let len = file.metadata().await.unwrap().len();

        let mut h = Hasher::new();
        h.read_from(len, file).await.unwrap();

        assert_eq!(h.digest(), digest([content.as_slice()]));
    }

    #[tokio::test]
    async fn async_reader_errors_on_early_eof() {
        let content = b"short".to_vec();
        let mut h = Hasher::new();
        assert!(h.read_from(content.len() as u64 + 1, io::Cursor::new(&content)).await.is_err());
    }
}
