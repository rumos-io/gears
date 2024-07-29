use darling::{util::PathList, FromAttributes, FromDeriveInput};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod new;
mod raw;

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(proto))]
struct ProtobufArg {
    raw: Option<syn::Type>,
    #[darling(default)]
    derive: PathList,
}

#[proc_macro_derive(Protobuf, attributes(proto))]
pub fn message_derive(input: TokenStream) -> TokenStream {
    expand_macro(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_macro(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ProtobufArg {
        raw,
        derive: raw_derives,
    } = ProtobufArg::from_derive_input(&input)?;

    match raw {
        Some(raw) => crate::raw::expand_raw_existing(raw, input),
        None => crate::new::extend_new_structure(input, raw_derives),
    }
}


#[derive(FromAttributes, Default)]
#[darling(default, attributes(proto), forward_attrs(allow, doc, cfg))]
struct RawProtobufAttr {
    name: Option<syn::Ident>,
}


fn is_option(ty: &syn::Type) -> bool {
    let opt = match ty {
        syn::Type::Path(typepath) if typepath.qself.is_none() => Some(typepath.path.clone()),
        _ => None,
    };

    if let Some(o) = opt {
        check_for_option(&o).is_some()
    } else {
        false
    }
}

fn check_for_option(path: &syn::Path) -> Option<&syn::PathSegment> {
    let idents_of_path = path.segments.iter().fold(String::new(), |mut acc, v| {
        acc.push_str(&v.ident.to_string());
        acc.push(':');
        acc
    });
    vec!["Option:", "std:option:Option:", "core:option:Option:"]
        .into_iter()
        .find(|s| idents_of_path == *s)
        .and_then(|_| path.segments.last())
}
