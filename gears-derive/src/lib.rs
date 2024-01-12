use proc_macro::TokenStream;
use quote::quote;
use syn::{self};
use syner::Syner;

#[derive(Syner)]
struct Gears {
    pub url: String,
}

// TODO: rename to AppMessage or MessageRouter?
#[proc_macro_derive(Message, attributes(gears))]
pub fn message_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_message(&ast)
}

fn impl_message(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let data = &ast.data;

    match data {
        syn::Data::Struct(_) => panic!("Message can only be derived for enums"),
        syn::Data::Enum(enum_data) => {
            let get_signers = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
                quote! {
                    Self::#i(msg) => proto_messages::cosmos::tx::v1beta1::Message::get_signers(msg)
                }
            });

            let validate_basic = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
                quote! {
                    Self::#i(msg) => msg.validate_basic()
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
                        Ok(Self::#ident(Any::try_into(value)?))
                    }
                }
            });

            let gen = quote! {
                impl proto_messages::cosmos::tx::v1beta1::Message for #name {


                    fn get_signers(&self) -> Vec<&AccAddress> {

                        match self {
                            //Self::Bank(msg) => msg.get_signers(),
                            //Self::#variant_name(msg) => msg.get_signers(),
                            #(#get_signers),*
                        }
                    }

                    fn validate_basic(&self) -> std::result::Result<(), String> {
                        match self {
                            //Self::Bank(msg) => msg.validate_basic(),
                            #(#validate_basic),*
                        }
                    }

                    // fn get_signers(&self) -> Vec<&proto_types::AccAddress> {
                    //     vec![&proto_types::AccAddress::from_bech32(&"cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux").unwrap()]
                    // }

                    // fn validate_basic(&self) -> Result<(), String> {
                    //     Ok(())
                    // }
                }

                impl From<#name> for Any {
                    fn from(msg: #name) -> Self {
                        match msg {
                            #(#into_any),*
                        }
                    }
                }

                impl TryFrom<Any> for #name {
                    type Error = proto_messages::Error;

                    fn try_from(value: Any) -> Result<Self, Self::Error> {

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
        syn::Data::Union(_) => panic!("Message can only be derived for enums"),
    }
}

// #[derive(Debug, Clone, Serialize, Message)]
// #[serde(untagged)]
// pub enum Message2 {
//     Bank(bank::Message),
// }

// pub trait Message:
//     Serialize + Clone + Send + Sync + 'static + Into<Any> + TryFrom<Any, Error = Error>
// {
//     //fn decode(raw: &Any) -> Self; // TODO: could be From<Any>

//     fn get_signers(&self) -> Vec<&AccAddress>;

//     fn validate_basic(&self) -> Result<(), String>;
// }
