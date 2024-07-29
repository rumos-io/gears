use darling::{util::PathList, FromDeriveInput};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod new;
mod raw;

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(proto), forward_attrs(allow, doc, cfg))]
struct ProtobufArg {
    raw: Option<syn::Type>,
    #[darling(default)]
    raw_derives: PathList,
}

#[proc_macro_derive(Protobuf, attributes(proto))]
pub fn message_derive(input: TokenStream) -> TokenStream {
    expand_macro(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_macro(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ProtobufArg { raw, raw_derives } = ProtobufArg::from_derive_input(&input)?;

    match raw {
        Some(raw) => crate::raw::expand_raw_existing(raw, input),
        None => crate::new::extend_new_structure(input, raw_derives),
    }
}
