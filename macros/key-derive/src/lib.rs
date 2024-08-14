#![cfg(not(doctest))]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/","Readme.md"))]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod params_key;
mod store_key;

#[cfg(not(doctest))]
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/","params.md"))]
#[proc_macro_derive(ParamsKeys, attributes(pkey))]
pub fn params_derive(input: TokenStream) -> TokenStream {
    params_key::expand_params(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(not(doctest))]
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/","store.md"))]
#[proc_macro_derive(StoreKeys, attributes(skey))]
pub fn store_derive(input: TokenStream) -> TokenStream {
    store_key::expand_store(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
