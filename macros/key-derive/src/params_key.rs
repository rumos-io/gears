use std::collections::HashSet;

use darling::{FromAttributes, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, DataEnum, DeriveInput, Variant};

use crate::KeysArg;

#[derive(FromAttributes, Default)]
#[darling(default, attributes(keys), forward_attrs(allow, doc, cfg))]
#[darling(and_then = Self::not_empty)]
pub struct KeysAttr {
    pub to_string: String,
}

impl KeysAttr {
    fn not_empty(self) -> darling::Result<Self> {
        if self.to_string.is_empty() {
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
            let mut set = HashSet::<String>::with_capacity(enum_variants.len());

            for Variant { attrs, ident, .. } in variants {
                let KeysAttr { to_string } = KeysAttr::from_attributes(&attrs)?;

                if !set.insert(to_string.clone()) {
                    Err(syn::Error::new(
                        ident.span(),
                        format!("Duplicate item: {}", to_string),
                    ))?
                }

                enum_variants.push(quote! { Self::#ident (_) => #to_string });
            }

            let result = quote! {
                impl #crate_prefix ::params::ParamsSubspaceKey for #ident
                {
                    fn name(&self) -> &'static str
                    {
                        math self{
                            #(#enum_variants),*
                        }
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
