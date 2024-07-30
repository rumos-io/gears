use proc_macro::TokenStream;
use syn::parse_macro_input;

mod raw;
mod proto;

#[proc_macro_derive(Protobuf, attributes(proto))]
pub fn proto_derive(input: TokenStream) -> TokenStream {
    proto::expand_raw_existing(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Raw, attributes(raw))]
pub fn raw_derive(input: TokenStream) -> TokenStream {
    raw::extend_new_structure(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
