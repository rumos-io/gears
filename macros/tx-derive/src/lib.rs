use darling::{util::Flag, FromAttributes, FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, Path};

mod enum_impl;
mod struct_impl;

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(msg))]
struct MessageArg {
    #[darling(default)]
    pub gears: Flag,
    #[darling(default)]
    pub url: String,
    #[darling(default)]
    pub amino_url: Option<String>,
}

#[derive(FromAttributes, Default)]
#[darling(default, attributes(msg), forward_attrs(allow, doc, cfg))]
struct MessageAttr {
    #[darling(default)]
    pub url: Option<PathOrString>,
    #[darling(default)]
    pub signer: Flag,
}

#[derive(Debug, FromMeta)]
enum PathOrString {
    Path(Path),
    String(String),
}

impl ToTokens for PathOrString {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            PathOrString::Path(var) => var.to_tokens(tokens),
            PathOrString::String(var) => var.to_tokens(tokens),
        }
    }
}

#[proc_macro_derive(AppMessage, attributes(msg))]
pub fn message_derive(input: TokenStream) -> TokenStream {
    inner::expand_macro(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

mod inner {
    use darling::FromDeriveInput;
    use quote::quote;
    use syn::DeriveInput;

    use crate::{enum_impl, struct_impl, MessageArg};

    pub fn expand_macro(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
        let MessageArg {
            gears,
            url,
            amino_url,
        } = MessageArg::from_derive_input(&input)?;
        let DeriveInput { ident, data, .. } = input;

        let crate_prefix = match gears.is_present() {
            true => quote! { crate },
            false => quote! { ::gears },
        };

        match data {
            syn::Data::Struct(data) => {
                struct_impl::expand_macro(data, ident, crate_prefix, url, amino_url)
            }
            syn::Data::Enum(data) => enum_impl::expand_macro(data, ident, crate_prefix),
            syn::Data::Union(_) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "TODO can't be derived for `Union`",
            )),
        }
    }
}
