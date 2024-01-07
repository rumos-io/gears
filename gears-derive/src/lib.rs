use proc_macro::TokenStream;
use quote::quote;
use syn;

// TODO: rename to AppMessage or MessageRouter?
#[proc_macro_derive(Message)]
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
            // for variant in enum_data.variants.iter() {
            //     let variant_name = &variant.ident;
            //     let variant_fields = &variant.fields;

            //     match variant_fields {
            //         syn::Fields::Named(fields_named) => {
            //             let named_field = fields_named.named.first().unwrap();
            //         }
            //         syn::Fields::Unnamed(_) => {
            //             panic!("Message can only be derived for enums with named fields")
            //         }
            //         syn::Fields::Unit => {
            //             panic!("Message can only be derived for enums with named fields")
            //         }
            //     }
            // }

            //let variant_name = enum_data.variants.first().unwrap().clone().ident;

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
            //let variant_name = "Blah";

            //for variant in enum_data.variants.iter() {}

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
