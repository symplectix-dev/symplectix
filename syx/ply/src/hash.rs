//! Digest.

use std::fmt;
use std::fmt::Write as _;
use std::io::{
    self,
    Read,
};

use sha2::Digest as _;
use tokio::io::{
    AsyncRead,
    AsyncReadExt as _,
};

/// Read size for streaming a reader's bytes into a digest,
/// so a large part is never buffered whole in memory.
const BUF_SIZE: usize = 1 << 16;

/// Builds a length-prefixed digest over an ordered sequence of parts.
///
/// This framing is self-delimiting, so no two distinct sequences
/// of parts produce the same digest. For example,
/// `part(b"a").part(b"b")` cannot collide with `part(b"ab")`.
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
    /// `r` is trusted to be exactly `len` bytes; a plain `Read` cannot
    /// report its own length up front without being fully consumed, so the
    /// caller must already know it. Returns an error if `r` runs out
    /// before `len` bytes are read; does not check for extra trailing
    /// bytes in `r` beyond `len`.
    pub fn reader(&mut self, len: u64, mut r: impl Read) -> io::Result<&mut Self> {
        self.hasher.update(len.to_be_bytes());

        let mut remaining = len;
        let mut buf = [0u8; BUF_SIZE];
        while remaining > 0 {
            let want = remaining.min(BUF_SIZE as u64) as usize;
            let n = r.read(&mut buf[..want])?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!("reader ended {remaining} bytes short of the declared length {len}"),
                ));
            }
            self.hasher.update(&buf[..n]);
            remaining -= n as u64;
        }
        Ok(self)
    }

    /// Async counterpart to `reader`: fold a part of known `len` bytes,
    /// read from `r`, into the digest, without needing a blocking thread.
    pub async fn async_reader(
        &mut self,
        len: u64,
        mut r: impl AsyncRead + Unpin,
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
            remaining -= n as u64;
        }
        Ok(self)
    }

    /// Finalize and return the digest's bytes.
    pub fn digest(self) -> Digest {
        Digest(self.hasher.finalize().into())
    }
}

/// Digest of `value`'s canonical CBOR encoding (RFC 8949 deterministic
/// encoding: smallest integer forms, definite-length items, sorted map
/// keys).
pub(crate) fn digest_of<T: serde::Serialize>(value: &T) -> Digest {
    let mut h = Hasher::new();
    h.part(
        // Plain `cbor2::to_vec` is not guaranteed deterministic (RFC 8949 allows
        // non-canonical encodings of the same value), so this must go through
        // `to_canonical_vec` specifically.
        cbor2::to_canonical_vec(value).expect("serializing to CBOR should not fail"),
    );
    h.digest()
}

/// A digest's raw bytes.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Digest(#[serde(with = "serde_bytes")] [u8; 32]);

impl Digest {
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

impl From<[u8; 32]> for Digest {
    fn from(bytes: [u8; 32]) -> Self {
        Digest(bytes)
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

#[cfg(test)]
mod tests {
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
    fn digest_byte_buf() {
        let d = digest([b"hello"]);
        let d_bytes = serde_bytes::ByteBuf::from(d.as_ref());
        assert_eq!(cbor2::to_vec(&d_bytes).unwrap(), cbor2::to_vec(&d).unwrap());
    }

    #[test]
    fn digest_round_trips_through_cbor() {
        let want = digest([b"hello"]);
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got: Digest = cbor2::from_slice(&bytes).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn digest_from_array_round_trips() {
        let want = digest([b"hello"]);
        let bytes: [u8; 32] = want.as_ref().try_into().unwrap();
        assert_eq!(Digest::from(bytes), want);
    }

    #[derive(serde::Serialize)]
    struct Example {
        name:  String,
        count: u32,
    }

    #[test]
    fn digest_of_is_deterministic() {
        let a = Example { name: "foo".to_string(), count: 1 };
        let b = Example { name: "foo".to_string(), count: 1 };
        assert_eq!(digest_of(&a), digest_of(&b));
    }

    #[test]
    fn digest_of_depends_on_every_field() {
        let base = Example { name: "foo".to_string(), count: 1 };
        let other_name = Example { name: "bar".to_string(), count: 1 };
        let other_count = Example { name: "foo".to_string(), count: 2 };
        assert_ne!(digest_of(&base), digest_of(&other_name));
        assert_ne!(digest_of(&base), digest_of(&other_count));
    }

    #[test]
    fn digest_of_matches_canonical_cbor_hash() {
        let value = Example { name: "foo".to_string(), count: 1 };
        let bytes = cbor2::to_canonical_vec(&value).unwrap();
        let mut h = Hasher::new();
        h.part(bytes);
        assert_eq!(digest_of(&value), h.digest());
    }

    #[test]
    fn deterministic() {
        assert_eq!(digest([b"hello".as_slice()]), digest([b"hello".as_slice()]));
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
    fn hex_is_lowercase_64_chars() {
        let hex = format!("{:x}", digest([b"hello".as_slice()]));
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn hex_matches_hash_bytes() {
        let want: String =
            digest([b"hello".as_slice()]).0.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(format!("{:x}", digest([b"hello".as_slice()])), want);
    }

    #[test]
    fn sharded_hex_depth_zero_is_plain_hex() {
        let d = digest([b"hello".as_slice()]);
        assert_eq!(d.hex(0), format!("{d:x}"));
    }

    #[test]
    fn sharded_hex_splits_leading_bytes() {
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

    #[test]
    fn reader_matches_in_memory_bytes() {
        let content = b"hello, reader".to_vec();

        let mut h = Hasher::new();
        h.reader(content.len() as u64, io::Cursor::new(&content)).unwrap();

        assert_eq!(h.digest(), digest([content.as_slice()]));
    }

    #[test]
    fn reader_is_chunked_not_buffered_at_once() {
        // Content larger than one BUF_SIZE read still hashes correctly,
        // proving the reader loops instead of assuming a single read call
        // drains everything.
        let content = vec![0x42u8; BUF_SIZE * 2 + 1];

        let mut h = Hasher::new();
        h.reader(content.len() as u64, io::Cursor::new(&content)).unwrap();

        assert_eq!(h.digest(), digest([content.as_slice()]));
    }

    #[test]
    fn reader_works_with_a_real_file() {
        // A real fs::File (not just an in-memory Cursor) implements Read
        // the same way; the caller supplies the length via `stat()`.
        let path = testing::rlocation("_main/.rustfmt.toml");
        let content = std::fs::read(&path).unwrap();
        let file = std::fs::File::open(&path).unwrap();
        let len = file.metadata().unwrap().len();

        let mut h = Hasher::new();
        h.reader(len, file).unwrap();

        assert_eq!(h.digest(), digest([content.as_slice()]));
    }

    #[test]
    fn reader_errors_on_early_eof() {
        let content = b"short".to_vec();
        let mut h = Hasher::new();
        assert!(h.reader(content.len() as u64 + 1, io::Cursor::new(&content)).is_err());
    }
}
