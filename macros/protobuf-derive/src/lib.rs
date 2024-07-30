use darling::{util::PathList, FromAttributes, FromDeriveInput};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod new;
mod raw;

#[proc_macro_derive(Protobuf, attributes(proto))]
pub fn message_derive(input: TokenStream) -> TokenStream {
    raw::expand_raw_existing(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Raw, attributes(raw))]
pub fn message_derive(input: TokenStream) -> TokenStream {
    new::extend_new_structure(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
