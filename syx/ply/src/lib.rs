//! Digest.

use sha2::{Digest as _, Sha256};

/// Builds a digest over an ordered sequence of parts.
///
/// Each part is folded in as an 8-byte big-endian length prefix followed by
/// its bytes. This framing is self-delimiting, so no two distinct sequences
/// of parts produce the same digest (e.g. `var(b"a").var(b"b")` cannot
/// collide with `var(b"ab")`).
pub struct Digest {
    hasher: Sha256,
}

impl Digest {
    pub fn new() -> Self {
        Digest { hasher: Sha256::new() }
    }

    /// Fold one more part into the digest.
    pub fn var(&mut self, part: impl AsRef<[u8]>) -> &mut Self {
        let bytes = part.as_ref();
        self.hasher.update((bytes.len() as u64).to_be_bytes());
        self.hasher.update(bytes);
        self
    }

    /// Finalize and return the raw digest bytes.
    pub fn hash(self) -> Vec<u8> {
        self.hasher.finalize().to_vec()
    }
}

impl Default for Digest {
    fn default() -> Self {
        Self::new()
    }
}

/// Digest of `parts`, combined in order: equal parts (in the same order)
/// always give the same digest, different parts almost surely give
/// different ones. Pass content alone to content-address it, or fold in
/// metadata parts (e.g. `digest([content, b"ocr-v3"])`) to address that too.
pub fn digest<I, T>(parts: I) -> Vec<u8>
where
    I: IntoIterator<Item = T>,
    T: AsRef<[u8]>,
{
    let mut d = Digest::new();
    for part in parts {
        d.var(part);
    }
    d.hash()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut d = Digest::new();
        d.var(b"a").var(b"b");
        assert_eq!(d.hash(), digest([b"a".as_slice(), b"b".as_slice()]));
    }

    #[test]
    fn empty_parts_is_stable() {
        assert_eq!(digest(Vec::<&[u8]>::new()), digest(Vec::<&[u8]>::new()));
    }
}
