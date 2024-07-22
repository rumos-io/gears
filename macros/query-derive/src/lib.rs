use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::parse_macro_input;

use quote::quote;
use syn::DeriveInput;

#[derive(FromDeriveInput)]
#[darling(attributes(query), forward_attrs(allow, doc, cfg))]
struct QueryAttr {
    pub kind: String,
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
        syn::Data::Struct(_) => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Query can only be derived for enums",
        )),
        syn::Data::Union(_) => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Query can only be derived for enums",
        )),
        syn::Data::Enum(enum_data) => match QueryAttr::from_derive_input(&input)?.kind.as_str() {
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
                    impl  ::gears::types::query::Query for #ident {
                        fn query_url(&self) -> &'static str  {
                            match self {
                                #(#query_url),*
                            }
                        }

                        fn into_bytes(self) -> std::vec::Vec<u8> {
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
                        Self::#i(q) => q.encode_vec().expect("Should be okay. In future versions of IBC they removed Result")
                    }
                });

                let gen = quote! {
                    impl  ::gears::baseapp::query::QueryResponse for #ident {
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
        },
    }
}
