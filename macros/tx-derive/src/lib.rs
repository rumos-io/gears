use darling::{util::Flag, FromAttributes, FromDeriveInput};
use proc_macro::TokenStream;
use syn::{parse_macro_input, spanned::Spanned};

mod enum_impl;
mod struct_impl;

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(tx_msg))]
struct MessageArg {
    #[darling(default)]
    pub gears: Flag,
}

#[derive(FromAttributes, Default)]
#[darling(default, attributes(tx_msg), forward_attrs(allow, doc, cfg))]
#[darling(and_then = Self::validate)]
struct MessageAttr {
    pub url: Option<String>,
}

impl MessageAttr {
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

    use crate::{enum_impl, MessageArg};

    pub fn expand_macro(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
        let MessageArg { gears } = MessageArg::from_derive_input(&input)?;
        let DeriveInput { ident, data, .. } = input;

        let crate_prefix = match gears.is_present() {
            true => quote! { crate },
            false => quote! { ::gears },
        };

        match data {
            syn::Data::Struct(_) => Ok(quote! {}),
            syn::Data::Enum(data) => enum_impl::expand_macro(data, ident, crate_prefix),
            syn::Data::Union(_) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Query can't be derived for `Union`",
            )),
        }
    }
}

// fn impl_message(ast: &syn::DeriveInput) -> TokenStream {
//     let name = &ast.ident;

//     let data = &ast.data;

//     match data {
//         syn::Data::Struct(_) => panic!("Message can only be derived for enums"),
//         syn::Data::Union(_) => panic!("Message can only be derived for enums"),
//         syn::Data::Enum(enum_data) => {
//             let get_signers = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
//                 quote! {
//                     Self::#i(msg) => ::gears::types::tx::TxMessage::get_signers(msg)
//                 }
//             });

//             let type_url = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
//                 quote! {
//                     Self::#i(msg) => ::gears::types::tx::TxMessage::type_url(msg)
//                 }
//             });

//             let amino_url = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
//                 quote! {
//                     Self::#i(msg) => ::gears::types::tx::TxMessage::amino_url(msg)
//                 }
//             });

//             let into_any = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
//                 quote! {
//                     #name ::#i(msg) => msg.into()
//                 }
//             });

//             let from_any = enum_data.variants.iter().map(|v| {
//                 let attr = &v.attrs;
//                 let ident = &v.ident;

//                 let attrs = Gears::parse_attrs(attr).unwrap();
//                 let url = attrs.url;

//                 quote! {
//                     if value.type_url.starts_with(#url) {
//                         Ok(Self::#ident(::gears::core::any::google::Any::try_into(value)?))
//                     }
//                 }
//             });

//             let gen = quote! {
//                 impl  ::gears::types::tx::TxMessage for #name {

//                     fn get_signers(&self) -> Vec<&::gears::types::address::AccAddress> {

//                         match self {
//                             #(#get_signers),*
//                         }
//                     }

//                     fn type_url(&self) -> &'static str {
//                         match self {
//                             #(#type_url),*
//                         }
//                     }

//                     fn amino_url(&self) -> &'static str {
//                         match self {
//                             #(#amino_url),*
//                         }
//                     }

//                 }

//                 impl From<#name> for ::gears::core::any::google::Any {
//                     fn from(msg: #name) -> Self {
//                         match msg {
//                             #(#into_any),*
//                         }
//                     }
//                 }

//                 impl TryFrom<::gears::core::any::google::Any> for #name {
//                     type Error = ::gears::core::errors::CoreError;

//                     fn try_from(value: ::gears::core::any::google::Any) -> Result<Self, Self::Error> {

//                         #(#from_any) else*

//                          else {
//                             Err(::gears::core::errors::CoreError::DecodeGeneral(
//                                 "message type not recognized".into(),
//                             ))
//                         }
//                     }
//                 }

//             };
//             gen.into()
//         }
//     }
// }
