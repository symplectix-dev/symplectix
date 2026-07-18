//! Shared fixtures for `cas`'s external test suite.
// `shared_srcs` compiles this file into every test binary in the
// suite separately, so not every item here is used by every one.
#![allow(dead_code)]

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Example {
    pub name:  String,
    pub count: u32,
}

impl cas::ToBytes for Example {
    type Error = cbor2::ser::Error;

    fn to_bytes(&self) -> Result<cas::Bytes, Self::Error> {
        cbor2::to_canonical_vec(self).map(cas::Bytes::from)
    }
}

impl cas::FromBytes for Example {
    type Error = cbor2::de::Error;

    fn from_bytes(bytes: cas::Bytes) -> Result<Self, Self::Error> {
        cbor2::from_slice(&bytes)
    }
}

pub fn digest_bytes(bytes: &[u8]) -> cas::Digest {
    let mut h = cas::Hasher::new();
    h.part(bytes);
    h.digest()
}

/// Digest of `parts`, combined in order.
pub fn digest_parts<I, T>(parts: I) -> cas::Digest
where
    I: IntoIterator<Item = T>,
    T: AsRef<[u8]>,
{
    let mut h = cas::Hasher::new();
    h.parts(parts);
    h.digest()
}

pub fn store() -> (testing::TempDir, cas::Store) {
    let dir = testing::tempdir();
    let store = cas::Store::open(dir.path()).unwrap();
    (dir, store)
}

/// `cas::digest`, unwrapped: every `ToBytes` impl used in this suite is
/// expected to succeed, so tests don't need to handle the error case.
pub fn digest<T: cas::ToBytes>(value: &T) -> cas::Digest {
    cas::digest(value).unwrap()
}
