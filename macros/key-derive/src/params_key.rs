use std::collections::HashSet;

use darling::{util::Flag, FromAttributes};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, DeriveInput, Variant};

use crate::expand::KeysAttr;

pub fn expand_params(
    DeriveInput { ident, data, .. }: DeriveInput,
    gears: Flag,
) -> syn::Result<TokenStream> {
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
