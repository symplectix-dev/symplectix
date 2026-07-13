//! Action: a content-addressed reference to something runnable.

use crate::hash::{
    self,
    Digest,
    Hasher,
};

/// A content-addressed reference to something runnable:
/// a function and the manifest to invoke it with, each referenced by
/// digest rather than embedded. Identical function + manifest always produce
/// the same `Action` digest, so identical runs dedup, and any change to
/// either input changes it, so a recorded digest is tamper-evident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Action {
    /// Digest of the `Function` to run.
    pub func: Digest,
    /// Digest of the `Manifest` to run the function with.
    pub conf: Digest,
}

impl Action {
    /// Digest of the action itself, combining `func` and `conf` in order.
    pub fn digest(&self) -> Digest {
        hash::digest_of(self)
    }
}

impl From<Action> for Digest {
    fn from(action: Action) -> Self {
        action.digest()
    }
}

impl TryFrom<&[u8]> for Action {
    type Error = cbor2::de::Error;

    /// Deserialize `bytes` (a CBOR encoding, canonical or not) as an
    /// `Action`.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        cbor2::from_slice(bytes)
    }
}

/// Something an `Action` can run, referenced by digest. Serde's enum
/// encoding tags each variant by name, so two `Function`s that happen to
/// wrap the same inner digest but mean different things (e.g. an OCI image
/// vs. some other artifact format) never fold into the same bytes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Function {
    /// An OCI image, referenced by its manifest digest.
    Oci(Digest),
}

impl Function {
    /// Digest of the function reference itself.
    pub fn digest(&self) -> Digest {
        hash::digest_of(self)
    }
}

impl From<Function> for Digest {
    fn from(function: Function) -> Self {
        function.digest()
    }
}

impl TryFrom<&[u8]> for Function {
    type Error = cbor2::de::Error;

    /// Deserialize `bytes` (a CBOR encoding, canonical or not) as a
    /// `Function`.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        cbor2::from_slice(bytes)
    }
}

/// The configuration to invoke an `Action`'s function with.
///
/// Shape still evolving -- currently just enough to be
/// content-addressable.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    /// The manifest's name.
    pub name: String,
}

impl Manifest {
    /// Digest of the config's content.
    pub fn digest(&self) -> Digest {
        hash::digest_of(self)
    }
}

impl From<Manifest> for Digest {
    fn from(manifest: Manifest) -> Self {
        manifest.digest()
    }
}

impl TryFrom<&[u8]> for Manifest {
    type Error = cbor2::de::Error;

    /// Deserialize `bytes` (a CBOR encoding, canonical or not) as a
    /// `Manifest`.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        cbor2::from_slice(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(bytes: &[u8]) -> Digest {
        let mut h = Hasher::new();
        h.part(bytes);
        h.digest()
    }

    #[test]
    fn manifest_digest_is_deterministic() {
        let a = Manifest { name: "foo".to_string() };
        let b = Manifest { name: "foo".to_string() };
        assert_eq!(a.digest(), b.digest());
    }

    #[test]
    fn manifest_digest_depends_on_name() {
        let a = Manifest { name: "a".to_string() };
        let b = Manifest { name: "b".to_string() };
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn function_oci_digest_is_deterministic() {
        let inner = digest(b"image");
        let a = Function::Oci(inner);
        let b = Function::Oci(inner);
        assert_eq!(a.digest(), b.digest());
    }

    #[test]
    fn function_oci_digest_depends_on_inner_digest() {
        let a = Function::Oci(digest(b"image-a"));
        let b = Function::Oci(digest(b"image-b"));
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn function_oci_digest_is_tagged_not_passthrough() {
        // The kind tag must actually be folded in, not just the inner
        // digest re-exposed unchanged.
        let inner = digest(b"image");
        assert_ne!(Function::Oci(inner).digest(), inner);
    }

    #[test]
    fn action_digest_is_deterministic() {
        let func = Function::Oci(digest(b"image")).digest();
        let conf = digest(b"manifest");
        let a = Action { func, conf };
        let b = Action { func, conf };
        assert_eq!(a.digest(), b.digest());
    }

    #[test]
    fn action_digest_changes_with_func() {
        let conf = digest(b"manifest");
        let a = Action { func: digest(b"func-a"), conf };
        let b = Action { func: digest(b"func-b"), conf };
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn action_digest_changes_with_conf() {
        let func = digest(b"func");
        let a = Action { func, conf: digest(b"manifest-a") };
        let b = Action { func, conf: digest(b"manifest-b") };
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn manifest_round_trips_through_cbor() {
        let want = Manifest { name: "foo".to_string() };
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got: Manifest = cbor2::from_slice(&bytes).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn function_round_trips_through_cbor() {
        let want = Function::Oci(digest(b"image"));
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got: Function = cbor2::from_slice(&bytes).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn action_round_trips_through_cbor() {
        let want = Action { func: digest(b"func"), conf: digest(b"conf") };
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got: Action = cbor2::from_slice(&bytes).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn action_try_from_bytes_round_trips() {
        let want = Action { func: digest(b"func"), conf: digest(b"conf") };
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got = Action::try_from(bytes.as_slice()).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn action_try_from_rejects_garbage_bytes() {
        assert!(Action::try_from(&b"not cbor"[..]).is_err());
    }

    #[test]
    fn manifest_try_from_bytes_round_trips() {
        let want = Manifest { name: "foo".to_string() };
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got = Manifest::try_from(bytes.as_slice()).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn manifest_try_from_rejects_garbage_bytes() {
        assert!(Manifest::try_from(&b"not cbor"[..]).is_err());
    }

    #[test]
    fn function_try_from_bytes_round_trips() {
        let want = Function::Oci(digest(b"image"));
        let bytes = cbor2::to_canonical_vec(&want).unwrap();
        let got = Function::try_from(bytes.as_slice()).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn function_try_from_rejects_garbage_bytes() {
        assert!(Function::try_from(&b"not cbor"[..]).is_err());
    }
}
