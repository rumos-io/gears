use proc_macro::TokenStream;
use quote::quote;
use syn::{self};
use syner::Syner;

#[derive(Syner)]
struct Gears {
    pub url: String,
}

// TODO: rename to AppMessage or MessageRouter?
#[proc_macro_derive(RoutingMessage, attributes(gears))]
pub fn message_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_message(&ast)
}

fn impl_message(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let data = &ast.data;

    match data {
        syn::Data::Struct(_) => panic!("Message can only be derived for enums"),
        syn::Data::Union(_) => panic!("Message can only be derived for enums"),
        syn::Data::Enum(enum_data) => {
            let get_signers = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
                quote! {
                    Self::#i(msg) => ::gears::types::tx::TxMessage::get_signers(msg)
                }
            });

            let type_url = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
                quote! {
                    Self::#i(msg) => ::gears::types::tx::TxMessage::type_url(msg)
                }
            });

            let into_any = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
                quote! {
                    #name ::#i(msg) => msg.into()
                }
            });

            let from_any = enum_data.variants.iter().map(|v| {
                let attr = &v.attrs;
                let ident = &v.ident;

                let attrs = Gears::parse_attrs(attr).unwrap();
                let url = attrs.url;

                quote! {
                    if value.type_url.starts_with(#url) {
                        Ok(Self::#ident(::gears::core::any::google::Any::try_into(value)?))
                    }
                }
            });

            let gen = quote! {
                impl  ::gears::types::tx::TxMessage for #name {

                    fn get_signers(&self) -> Vec<&::gears::types::address::AccAddress> {

                        match self {
                            #(#get_signers),*
                        }
                    }

                    fn type_url(&self) -> &'static str {
                        match self {
                            #(#type_url),*
                        }
                    }

                }

                impl From<#name> for ::gears::core::any::google::Any {
                    fn from(msg: #name) -> Self {
                        match msg {
                            #(#into_any),*
                        }
                    }
                }

                impl TryFrom<::gears::core::any::google::Any> for #name {
                    type Error = ::gears::core::errors::CoreError;

                    fn try_from(value: ::gears::core::any::google::Any) -> Result<Self, Self::Error> {

                        #(#from_any) else*

                         else {
                            Err(::gears::core::errors::CoreError::DecodeGeneral(
                                "message type not recognized".into(),
                            ))
                        }
                    }
                }


            };
            gen.into()
        }
    }
}
