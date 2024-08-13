use std::collections::HashSet;

use darling::{util::Flag, FromAttributes, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, DeriveInput, Variant};

use crate::KeysAttr;

#[derive(FromDeriveInput)]
#[darling(attributes(pkey))]
struct KeysArg {
    #[darling(default)]
    pub gears: Flag,
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

                if !set.insert(to_string.clone()) {
                    Err(syn::Error::new(
                        ident.span(),
                        format!("Duplicate item: {}", to_string),
                    ))?
                }

                enum_variants.push(quote! { Self::#ident => #to_string });
                from_str_impls.push(quote! { #to_string => Self::#ident });
            }

            let result = quote! {
                impl #crate_prefix ::params::ParamsSubspaceKey for #ident
                {
                    fn name(&self) -> &'static str
                    {
                        match self{
                            #(#enum_variants),*
                        }
                    }

                    fn from_subspace_str(val: &str) -> ::std::result::Result<Self, #crate_prefix::params::SubspaceParseError> {
                        let result = match val
                        {
                            #(#from_str_impls),*
                            , _ => ::std::result::Result::Err(#crate_prefix::params::SubspaceParseError(::std::format!("missing valid key: {val} not found")))?,
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
