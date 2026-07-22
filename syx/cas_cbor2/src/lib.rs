//! Derives `cas::ToBytes`/`cas::FromBytes` via canonical CBOR, using the
//! target type's own `Serialize`/`Deserialize` impls. This crate's own
//! choice of encoding; `cas` itself has no opinion on it.

use quote::quote;
use syn::{
    parse_macro_input,
    DeriveInput,
};

#[proc_macro_derive(ToBytes)]
pub fn derive_to_bytes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::cas::ToBytes for #ident #ty_generics #where_clause {
            type Error = ::cbor2::ser::Error;

            fn to_bytes(&self) -> Result<::cas::Bytes, Self::Error> {
                // Plain `cbor2::to_vec` is not guaranteed deterministic (RFC 8949
                // allows non-canonical encodings of the same value), so this must
                // go through `to_canonical_vec` specifically.
                ::cbor2::to_canonical_vec(self).map(::cas::Bytes::from)
            }
        }
    }
    .into()
}

#[proc_macro_derive(FromBytes)]
pub fn derive_from_bytes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::cas::FromBytes for #ident #ty_generics #where_clause {
            type Error = ::cbor2::de::Error;

            fn from_bytes(bytes: ::cas::Bytes) -> Result<Self, Self::Error> {
                ::cbor2::from_slice(&bytes)
            }
        }
    }
    .into()
}
