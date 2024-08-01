#![cfg(not(doctest))]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/","Readme.md"))]

use darling::{util::Flag, FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use syn::parse_macro_input;

use quote::quote;
use syn::DeriveInput;

#[derive(FromMeta, Default)]
#[darling(and_then = Self::not_both)]
struct RequestOrResponse {
    request: Flag,
    response: Flag,
}

impl RequestOrResponse {
    fn not_both(self) -> darling::Result<Self> {
        match (self.request.is_present(), self.response.is_present()) {
            (true, true) => Err(
                darling::Error::custom("Cannot set `request` and `response`")
                    .with_span(&self.response.span()),
            ),
            _ => Ok(self),
        }
    }
}
enum Kind {
    Request,
    Response,
}

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(query), forward_attrs(allow, doc, cfg))]
#[darling(supports(struct_any, enum_tuple, enum_newtype))]
struct QueryAttr {
    #[darling(flatten, default)]
    pub kind: RequestOrResponse,
    pub url: Option<String>,
}

/// Generates impl for Query trait and add Protobuf.
///
/// _Note_: you still need to implement `From<Self> for Raw` and `TryFrom<Raw> for Self` manually
#[proc_macro_derive(Query, attributes(query))]
pub fn message_derive(input: TokenStream) -> TokenStream {
    expand_macro(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_macro(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let DeriveInput { ident, data, .. } = &input;
    let QueryAttr { kind, url } = QueryAttr::from_derive_input(&input)?;

    fn error() -> syn::Result<Kind> {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Invalid `kind`. Possible values: request, response".to_string(),
        ))
    }

    let kind = match (kind.request.is_present(), kind.response.is_present()) {
        (true, true) => unreachable!("We validated structure for such cases"),
        (true, false) => Kind::Request,
        (false, true) => Kind::Response,
        (false, false) => {
            if ident.to_string().to_lowercase().contains("request") {
                Kind::Request
            } else if ident.to_string().to_lowercase().contains("response") {
                Kind::Response
            } else {
                error()?
            }
        }
    };

    match data {
        syn::Data::Struct(_) => match kind {
            Kind::Request => {
                let url = match url {
                    Some(url) => quote! {
                        impl #ident
                        {
                           pub const QUERY_URL : &'static str = #url;
                        }
                    },
                    None => Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        "Request query requires `url` attribute".to_string(),
                    ))?,
                };

                let query_trait = quote! {
                    impl  ::gears::baseapp::Query for #ident {
                        fn query_url(&self) -> &'static str  {
                            Self::QUERY_URL
                        }

                        fn into_bytes(self) -> ::std::vec::Vec<u8> {
                            ::gears::core::Protobuf::encode_vec(&self)
                        }
                    }
                };

                let gen = quote! {
                    #query_trait

                    #url
                };

                Ok(gen)
            }
            Kind::Response => {
                let url = match url {
                    Some(_) => quote! {
                        impl #ident
                        {
                           pub const QUERY_URL : &'static str = #url;
                        }
                    },
                    None => quote! {},
                };

                let trait_impl = quote! {
                    impl  ::gears::baseapp::QueryResponse for #ident {
                        fn into_bytes(self) -> std::vec::Vec<u8> {
                            gears::core::Protobuf::encode_vec(&self)
                        }
                    }
                };

                let gen = quote! {

                    #url

                    #trait_impl
                };

                Ok(gen)
            }
        },
        syn::Data::Union(_) => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Query can't be derived for `Union`",
        )),
        // TODO: Support for enums with other enums
        syn::Data::Enum(enum_data) => {
            if url.is_some() {
                Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "Enum couldn't contain `url` attribute".to_string(),
                ))?
            }

            match kind {
                Kind::Request => {
                    let query_url = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
                        quote! {
                            Self::#i(q) => q.query_url()
                        }
                    });

                    let into_bytes = enum_data.variants.iter().map(|v| v.clone().ident).map(|i| {
                        quote! {
                            Self::#i(q) => ::gears::core::Protobuf::encode_vec(&q)
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

                    Ok(gen)
                }
                Kind::Response => {
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

                    Ok(gen)
                }
            }
        }
    }
}
