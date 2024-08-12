use std::collections::HashSet;

use darling::{util::Flag, FromAttributes, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, DeriveInput, Ident, Variant};

use crate::KeysAttr;

#[derive(FromDeriveInput)]
#[darling(attributes(skey))]
struct KeysArg {
    #[darling(default)]
    pub gears: Flag,
    pub params: Ident,
}

pub fn expand_store(input: DeriveInput) -> syn::Result<TokenStream> {
    let KeysArg { gears, params } = KeysArg::from_derive_input(&input)?;
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
                        math self {
                            #(#enum_variants),*
                        }
                    }

                    fn params() -> &'static Self {
                        const PARAM_KEY: Self = Self::#params;

                        &PARAM_KEY
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
