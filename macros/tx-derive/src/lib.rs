use darling::{util::Flag, FromAttributes, FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use syn::{parse_macro_input, spanned::Spanned};

mod enum_impl;
mod struct_impl;

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(tx_msg))]
struct MessageArg {
    #[darling(default)]
    pub gears: Flag,
    #[darling(default)]
    pub url: Url,
    #[darling(default)]
    pub amino_url: Url,
}

#[derive(FromAttributes, Default)]
#[darling(default, attributes(tx_msg), forward_attrs(allow, doc, cfg))]
struct MessageAttr {
    #[darling(default, flatten)]
    pub url: Url,
    #[darling(default)]
    pub signer: Flag,
}

#[derive(FromMeta, Default)]
#[darling(and_then = Self::validate)]
struct Url {
    url: Option<String>,
}

impl Url {
    fn validate(self) -> darling::Result<Self> {
        match &self.url {
            Some(var) => {
                match var.is_empty() {
                    true => Err(darling::Error::custom("Cannot set `url` to empty")
                        .with_span(&self.url.span())),
                    false => Ok(self),
                }
            }
            None => Ok(self),
        }
    }

    fn into_inner(self) -> Option<String> {
        self.url
    }
}

#[proc_macro_derive(AppMessage, attributes(tx_msg))]
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
