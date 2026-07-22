//! Wires up `cas::ToBytes`/`cas::FromBytes` for this crate's types, via
//! their own `Serialize`/`Deserialize` impls, using canonical CBOR
//! encoding. This is `func`'s own choice of encoding for its own types,
//! not something `cas` needs an opinion on.

use crate::{
    Command,
    Function,
};

macro_rules! storable {
    ($ty:ty) => {
        impl cas::ToBytes for $ty {
            type Error = cbor2::ser::Error;

            fn to_bytes(&self) -> Result<cas::Bytes, Self::Error> {
                // Plain `cbor2::to_vec` is not guaranteed deterministic (RFC 8949
                // allows non-canonical encodings of the same value), so this must
                // go through `to_canonical_vec` specifically.
                cbor2::to_canonical_vec(self).map(cas::Bytes::from)
            }
        }

        impl cas::FromBytes for $ty {
            type Error = cbor2::de::Error;

            fn from_bytes(bytes: cas::Bytes) -> Result<Self, Self::Error> {
                cbor2::from_slice(&bytes)
            }
        }
    };
}

storable!(Command);
storable!(Function);
