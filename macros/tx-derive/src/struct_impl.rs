use darling::FromAttributes;
use quote::quote;
use syn::{spanned::Spanned, DataStruct, Field, Generics, Ident};

use crate::MessageAttr;

pub fn expand_macro(
    DataStruct { fields, .. }: DataStruct,
    type_ident: Ident,
    generics: Generics,
    crate_prefix: proc_macro2::TokenStream,
    url: String,
    amino_url: Option<String>,
) -> syn::Result<proc_macro2::TokenStream> {
    let url = match url.is_empty() {
        false => Ok(url),
        true => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "`url` attribute is required for structure",
        )),
    }?;

    let (amino_url, empty_amino) = match amino_url {
        Some(amino_url) => match amino_url.is_empty() {
            true => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`amino_url` attribute can't be empty",
            ))?,
            false => (
                quote! {  pub const AMINO_URL : &'static str = #amino_url; },
                false,
            ),
        },
        None => (quote! {}, true),
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let type_url_impl = quote! {
        impl #impl_generics #type_ident #ty_generics #where_clause
        {
           pub const TYPE_URL : &'static str = #url;
           #amino_url
        }
    };

    let ty_generics_fish = ty_generics.as_turbofish();
    let from_msg_to_any_impl = quote! {
        impl #impl_generics ::std::convert::From<#type_ident #ty_generics> for  #crate_prefix::core::any::google::Any #where_clause {
            fn from(msg: #type_ident #ty_generics) -> Self {
                #crate_prefix::core::any::google::Any {
                    type_url: #type_ident #ty_generics_fish ::TYPE_URL.to_string(),
                    value: #crate_prefix::core::Protobuf::encode_vec(&msg),
                }
            }
        }
    };

    let try_from_any_to_msg_impl = quote! {
        impl #impl_generics TryFrom<#crate_prefix::core::any::google::Any> for #type_ident #ty_generics #where_clause {
            type Error = #crate_prefix::core::errors::CoreError;

            fn try_from(value: #crate_prefix::core::any::google::Any) -> ::std::result::Result<Self, Self::Error> {

                use #crate_prefix::core::Protobuf;

                match ::std::string::String::as_str(&value.type_url)
                {
                    Self::TYPE_URL => {
                        let msg = Self::decode::<::prost::bytes::Bytes>(::std::convert::Into::into(value.value))
                         .map_err(|e| #crate_prefix::core::errors::CoreError::DecodeProtobuf(::std::string::ToString::to_string(&e)))?;

                        Ok(msg)
                    },
                      _ => Err( #crate_prefix::core::errors::CoreError::DecodeGeneral(
                        ::std::convert::Into::into("message type not recognized"),
                    ))
                }
            }
        }
    };

    let mut signers = Vec::new();
    for Field { attrs, ident, .. } in fields {
        let MessageAttr { url: _, signer } = MessageAttr::from_attributes(&attrs)?;

        if signer.is_present() {
            match ident {
                Some(ident) => signers.push(ident),
                None => Err(syn::Error::new(
                    ident.span(),
                    "Can't use `signer` on tuple structure",
                ))?,
            }
        }
    }

    let signers_impl = quote! {
        fn get_signers(&self) -> ::std::vec::Vec<&#crate_prefix::types::address::AccAddress> {
            ::std::vec![
                #(&self.#signers),*
            ]
        }
    };

    let amino_impl = match empty_amino {
        true => quote! {},
        false => quote! {
            fn amino_url(&self) -> &'static str {
                Self::AMINO_URL
            }
        },
    };

    let tx_message_impl = quote! {
        impl #impl_generics #crate_prefix::types::tx::TxMessage for #type_ident #ty_generics #where_clause
        {
            #signers_impl

            fn type_url(&self) -> &'static str {
                Self::TYPE_URL
            }

            #amino_impl
        }
    };

    Ok(quote! {
        #type_url_impl

        #from_msg_to_any_impl

        #try_from_any_to_msg_impl

        #tx_message_impl
    })
}
