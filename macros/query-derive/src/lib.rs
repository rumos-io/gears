use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::{parse_macro_input, Type};

use quote::quote;
use syn::DeriveInput;

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(query), forward_attrs(allow, doc, cfg))]
struct QueryAttr {
    pub kind: String,
    pub raw: Option<Type>,
    pub url: Option<String>,
}

#[proc_macro_derive(Query, attributes(query))]
pub fn message_derive(input: TokenStream) -> TokenStream {
    expand_macro(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_macro(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let DeriveInput { ident, data, .. } = &input;

    match data {
        syn::Data::Struct(_) => {
            let QueryAttr { kind, raw, url } = QueryAttr::from_derive_input(&input)?;

            // TODO:MAYBE support of other serialization?
            let protobuf = match raw {
                Some(protobuf) => quote! {
                    impl ::gears::tendermint::types::proto::Protobuf<#protobuf> for #ident {}
                },
                None => Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("Query requires `raw` attribute for serialization from protobuf"),
                ))?,
            };

            match kind.as_str() {
                "request" => {
                    let url = match url {
                        Some(url) => quote! {
                            impl #ident
                            {
                                const QUERY_URL : &'static str = #url;
                            }
                        },
                        None => Err(syn::Error::new(
                            proc_macro2::Span::call_site(),
                            format!("Request query requires `url` attribute"),
                        ))?,
                    };

                    let query_trait = quote! {
                        impl  ::gears::baseapp::Query for #ident {
                            fn query_url(&self) -> &'static str  {
                                Self::QUERY_URL
                            }

                            fn into_bytes(self) -> ::std::vec::Vec<u8> {
                                gears::tendermint::types::proto::Protobuf::encode_vec(&self).expect("Should be okay. In future versions of IBC they removed Result")
                            }
                        }
                    };

                    let gen = quote! {
                        #query_trait

                        #protobuf

                        #url
                    };

                    Ok(gen.into())
                }
                "response" => {
                    let url = match url {
                        Some(_) => quote! {
                            impl #ident
                            {
                                const QUERY_URL : &'static str = #url;
                            }
                        },
                        None => quote! {},
                    };

                    let trait_impl = quote! {
                        impl  ::gears::baseapp::QueryResponse for #ident {
                            fn into_bytes(self) -> std::vec::Vec<u8> {
                                gears::tendermint::types::proto::Protobuf::encode_vec(&self).expect("Should be okay. In future versions of IBC they removed Result")
                            }
                        }
                    };

                    let gen = quote! {
                        #protobuf

                        #url

                        #trait_impl
                    };

                    Ok(gen.into())
                }
                _ => Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("Invalid `kind`. Possible values: request, response"),
                ))?,
            }
        }
        syn::Data::Union(_) => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Query can't be derived for `Union`",
        )),
        syn::Data::Enum(enum_data) => {
            let QueryAttr { kind, raw: _, url } = QueryAttr::from_derive_input(&input)?;
            if url.is_some() {
                Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("Enum couldn't contain `url` attribute"),
                ))?
            }

            match kind.as_str() {
                "request" => {
                    let query_url = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
                        quote! {
                            Self::#i(q) => q.query_url()
                        }
                    });

                    let into_bytes = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
                                quote! {
                                    Self::#i(q) => q.encode_vec().expect("Should be okay. In future versions of IBC they removed Result")
                                }
                            });

                    let gen = quote! {
                        impl  ::gears::baseapp::Query for #ident {
                            fn query_url(&self) -> &'static str  {
                                match self {
                                    #(#query_url),*
                                }
                            }

                            fn into_bytes(self) -> ::std::vec::Vec<u8> {
                                match self {
                                    #(#into_bytes),*
                                }
                            }
                        }
                    };

                    Ok(gen.into())
                }
                "response" => {
                    let into_bytes = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
                        quote! {
                            Self::#i(q) => q.into_bytes()
                        }
                    });

                    let gen = quote! {
                        impl  ::gears::baseapp::QueryResponse for #ident {
                            fn into_bytes(self) -> std::vec::Vec<u8> {
                                match self {
                                    #(#into_bytes),*
                                }
                            }
                        }
                    };

                    Ok(gen.into())
                }
                _ => Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("Invalid `kind`. Possible values: request, response"),
                ))?,
            }
        }
    }
}
