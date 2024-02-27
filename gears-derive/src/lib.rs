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
                    Self::#i(msg) => proto_messages::cosmos::tx::v1beta1::message::Message::get_signers(msg)
                }
            });

            let validate_basic = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
                quote! {
                    Self::#i(msg) => msg.validate_basic()
                }
            });

            let type_url = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
                quote! {
                    Self::#i(msg) => proto_messages::cosmos::tx::v1beta1::message::Message::type_url(msg)
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
                        Ok(Self::#ident(::proto_messages::any::Any::try_into(value)?))
                    }
                }
            });

            let gen = quote! {
                impl proto_messages::cosmos::tx::v1beta1::message::Message for #name {


                    fn get_signers(&self) -> Vec<&AccAddress> {

                        match self {
                            #(#get_signers),*
                        }
                    }

                    fn validate_basic(&self) -> std::result::Result<(), String> {
                        match self {
                            #(#validate_basic),*
                        }
                    }

                    fn type_url(&self) -> &'static str {
                        match self {
                            #(#type_url),*
                        }
                    }


                }

                impl From<#name> for proto_messages::any::Any {
                    fn from(msg: #name) -> Self {
                        match msg {
                            #(#into_any),*
                        }
                    }
                }

                impl TryFrom<proto_messages::any::Any> for #name {
                    type Error = proto_messages::Error;

                    fn try_from(value: proto_messages::any::Any) -> Result<Self, Self::Error> {

                        #(#from_any) else*

                         else {
                            Err(proto_messages::Error::DecodeGeneral(
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
