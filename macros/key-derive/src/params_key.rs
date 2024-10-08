use std::collections::HashSet;

use darling::{util::Flag, FromAttributes, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, DataEnum, DeriveInput, Variant};

#[derive(FromDeriveInput)]
#[darling(attributes(pkey))]
struct KeysArg {
    #[darling(default)]
    pub gears: Flag,
}

#[derive(FromAttributes, Default)]
#[darling(default, attributes(pkey), forward_attrs(allow, doc, cfg))]
#[darling(and_then = Self::not_empty)]
struct KeysAttr {
    pub to_string: String,
}

impl KeysAttr {
    fn not_empty(self) -> darling::Result<Self> {
        if self.to_string.is_empty() || self.to_string.replace(' ', "").is_empty() {
            Err(darling::Error::custom("key can't be empty").with_span(&self.to_string.span()))
        } else {
            Ok(self)
        }
    }
}

pub fn expand_params(input: DeriveInput) -> syn::Result<TokenStream> {
    let KeysArg { gears } = KeysArg::from_derive_input(&input)?;
    let DeriveInput { ident, data, .. } = input;

    match data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let crate_prefix = match gears.is_present() {
                true => quote! { crate },
                false => quote! { ::gears },
            };

            let mut enum_variants = Vec::<TokenStream>::new();
            let mut from_str_impls = Vec::<TokenStream>::new();
            let mut set = HashSet::<String>::with_capacity(enum_variants.len());

            for Variant { attrs, ident, .. } in variants {
                let KeysAttr { to_string } = KeysAttr::from_attributes(&attrs)?;

                if let Some(prefix) =
                    set.iter()
                        .find(|this| match this.len().cmp(&to_string.len()) {
                            std::cmp::Ordering::Less => to_string.starts_with(*this),
                            std::cmp::Ordering::Equal => this == &&to_string,
                            std::cmp::Ordering::Greater => this.starts_with(&to_string),
                        })
                {
                    Err(syn::Error::new(
                        ident.span(),
                        format!("Key: {} is prefix of another item: {}", to_string, prefix),
                    ))?
                }

                let _ = set.insert(to_string.clone());

                enum_variants
                    .push(quote! { Self::#ident => ::std::borrow::ToOwned::to_owned(#to_string) });
                from_str_impls.push(quote! { #to_string => Self::#ident });
            }

            let result = quote! {
                impl #crate_prefix ::params::ParamsSubspaceKey for #ident
                {
                    fn name(&self) -> String
                    {
                        match self{
                            #(#enum_variants),*
                        }
                    }

                    fn from_subspace_str(val: impl ::std::convert::AsRef<str>) -> ::std::result::Result<Self, #crate_prefix::params::SubspaceParseError> {
                        let result = match ::std::convert::AsRef::as_ref(&val)
                        {
                            #(#from_str_impls),*
                            , _ => ::std::result::Result::Err(#crate_prefix::params::SubspaceParseError(::std::format!("missing valid key: {} not found", ::std::convert::AsRef::as_ref(&val) )))?,
                        };

                        ::std::result::Result::Ok(result)
                    }
                }
            };

            Ok(result)
        }
        _ => Err(syn::Error::new(
            ident.span(),
            "Can be derived only on enums",
        )),
    }
}
