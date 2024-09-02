use darling::FromAttributes;
use quote::quote;
use syn::{spanned::Spanned, DataStruct, Field, Ident};

use crate::{MessageAttr, Url};

pub fn expand_macro(
    DataStruct { fields, .. }: DataStruct,
    type_ident: Ident,
    crate_prefix: proc_macro2::TokenStream,
    url: Url,
    amino_url: Url,
) -> syn::Result<proc_macro2::TokenStream> {
    let url = match url.into_inner() {
        Some(url) => Ok(url),
        None => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "`url` attribute is required for structure",
        )),
    }?;

    let amino_url = match amino_url.into_inner() {
        Some(amino_url) => Ok(amino_url),
        None => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "`amino_url` attribute is required for structure",
        )),
    }?;

    let type_url_impl = quote! {
        impl #type_ident
        {
           pub const TYPE_URL : &'static str = #url;
           pub const AMINO_URL : &'static str = #amino_url;
        }
    };

    let from_msg_to_any_impl = quote! {
        impl ::std::conver::From<#type_ident> for  #crate_prefix::any::google::Any  {
            fn from(msg: #type_ident) -> Self {
                #crate_prefix::any::google::Any {
                    type_url: Self::TYPE_URL.to_string(),
                    value: #crate_prefix::core::Protobuf::encode_vec(&msg),
                }
            }
        }
    };

    let try_from_any_to_msg_impl = quote! {
        impl TryFrom<#crate_prefix::any::google::Any> for #type_ident {
            type Error = #crate_prefix::errors::CoreError;

            fn try_from(value: #crate_prefix::any::google::Any) -> ::std::result::Result<Self, Self::Error> {
                match ::std::string::String::as_str(&value.type_url)
                {
                    Self::TYPE_URL => {
                        let msg = <Self as ::prost::Message>::decode::<::prost::bytes::Bytes>(::std::convert::Into::into(value.value))
                         .map_err(|e| #crate_prefix::errors::CoreError::DecodeProtobuf(::std::string::ToString::to_string(&e)))?;

                        Ok(msg)
                    },
                      _ => Err( #crate_prefix::errors::CoreError::DecodeGeneral(
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
                #(#signers),*
            ]
        }
    };

    let type_urls_fns_impl = quote! {
        fn type_url(&self) -> &'static str {
            Self::TYPE_URL
        }

        fn amino_url(&self) -> &'static str {
            Self::AMINO_URL
        }
    };

    let tx_message_impl = quote! {
        #signers_impl

        #type_urls_fns_impl
    };

    Ok(quote! {
        #type_url_impl

        #from_msg_to_any_impl

        #try_from_any_to_msg_impl

        #tx_message_impl
    })
}
