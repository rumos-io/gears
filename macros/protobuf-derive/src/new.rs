use darling::{
    util::{Flag, PathList},
    FromAttributes, FromMeta,
};
use quote::quote;
use syn::{DataStruct, DeriveInput, Field, TypePath};

use crate::{is_option, RawProtobufAttr};

#[derive(FromMeta)]
#[darling(rename_all = "lowercase")]
enum Kind {
    Sint32,
    Int64,
    Uint32,
    Uint64,
    Bool,
    String,
    Bytes,
}

impl Kind {
    fn to_prost_token(&self) -> proc_macro2::TokenStream {
        match self {
            Kind::Sint32 => quote! { int32 },
            Kind::Int64 => quote! { int64 },
            Kind::Uint32 => quote! { uint32 },
            Kind::Uint64 => quote! { uint64 },
            Kind::Bool => quote! { r#bool },
            Kind::String => quote! { string },
            Kind::Bytes => quote! { bytes },
        }
    }
}

#[derive(FromMeta, Default)]
#[darling(and_then = Self::not_both)]
struct OptionalOrRequired {
    optional: Flag,
    repeated: Flag,
}

impl OptionalOrRequired {
    fn not_both(self) -> darling::Result<Self> {
        if self.optional.is_present() && self.repeated.is_present() {
            Err(
                darling::Error::custom("Cannot set `optional` and `repeated`")
                    .with_span(&self.repeated.span()),
            )
        } else {
            Ok(self)
        }
    }
}

#[derive(FromAttributes)]
#[darling(attributes(proto))]
struct ProtobufAttr {
    #[darling(default)]
    raw: Option<syn::Path>,
    #[darling(flatten, default)]
    opt: OptionalOrRequired,
    kind: Kind,
    #[darling(default)]
    tag: Option<u32>,
}

pub fn extend_new_structure(
    DeriveInput {
        ident, data, vis, ..
    }: DeriveInput,
    raw_derives: PathList,
) -> syn::Result<proc_macro2::TokenStream> {
    match data {
        syn::Data::Struct(DataStruct { fields, .. }) => {
            let mut raw_fields = Vec::new();
            let mut result_fields = Vec::with_capacity(fields.len());
            let mut counter = 1;

            for Field {
                attrs,
                vis,
                ident,
                ty,
                ..
            } in fields
            {
                let ProtobufAttr {
                    raw,
                    opt: OptionalOrRequired { optional, repeated },
                    kind,
                    tag,
                } = ProtobufAttr::from_attributes(&attrs)?;

                let raw = raw
                    .map(|path| syn::Type::Path(TypePath { qself: None, path }))
                    .unwrap_or(ty.clone());
                let tag = tag.unwrap_or(counter);

                let kind = kind.to_prost_token();

                let result = match (optional.is_present(), repeated.is_present()) {
                    (true, true) => unreachable!("we validated structure to omit such case"),
                    (true, false) => quote! {
                        #[prost( #kind, optional, tag = #tag )]
                        #vis #ident : ::std::option::Option<#raw>,
                    },
                    (false, true) => quote! {
                       #[prost( #kind, repeated, tag = #tag )]
                       #vis #ident : std::vec::Vec<#raw>,
                    },
                    (false, false) => quote! {
                        #[prost( #kind, required, tag = #tag )]
                        #vis #ident : #raw
                    },
                };

                result_fields.push(result);

                counter = tag;
                counter += 1;

                let RawProtobufAttr { name } = RawProtobufAttr::from_attributes(&attrs)?;

                raw_fields.push((
                    name,
                    ident.ok_or(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        "Can't derive on tuple structures",
                    ))?,
                    ty,
                ));
            }

            let (gen_struct, raw_ident) = {
                let new_name = syn::Ident::new(
                    &format!("Raw{}", ident.to_string()),
                    proc_macro2::Span::call_site(),
                );

                let raw_derives = match raw_derives.is_empty() {
                    true => quote! {},
                    false => quote! { #[derive(#(#raw_derives,)*)] },
                };

                (
                    quote! {

                        #[derive(::prost::Message)]
                        #raw_derives
                        #vis struct  #new_name
                        {
                            #(#result_fields),*
                        }
                    },
                    new_name,
                )
            };

            let get_from = {
                let protobuf_trait_impl = quote! {
                    impl ::gears::tendermint::types::proto::Protobuf<#raw_ident> for #ident {}
                };

                let from_fields_iter_gen =
                    raw_fields
                        .iter()
                        .map(|(other_name, field_ident, field_type)| {
                            let other_name = other_name.clone().unwrap_or(field_ident.clone());

                            match is_option(&field_type) {
                                true => {
                                    quote! {
                                        #other_name : match value.#field_ident
                                        {
                                            Some(var) => Some( ::std::convert::Into::into(var)),
                                            None => None,
                                        }
                                    }
                                }
                                false => quote! {
                                    #other_name : ::std::convert::Into::into(value.#field_ident)
                                },
                            }
                        });

                let from_impl = quote! {
                    impl ::std::convert::From<#ident> for #raw_ident {
                        fn from(value: #ident) -> Self {
                            Self
                            {
                                #(#from_fields_iter_gen),*
                            }
                        }
                    }
                };

                let try_from_fields_iter_gen = raw_fields.iter().map(
                    |(other_name, field_ident, field_type)| {
                        let other_name = other_name.clone().unwrap_or(field_ident.clone());

                        match is_option(&field_type) {
                            true => {
                                quote! {
                                    #field_ident : match value.#other_name {
                                        Some(var) => Some(::std::convert::TryFrom::try_from(var)?),
                                        None => None,
                                    }
                                }
                            }
                            false => quote! {
                                #field_ident : ::std::convert::TryFrom::try_from(value.#other_name)?
                            },
                        }
                    },
                );

                let try_from = quote! {

                    impl TryFrom<#raw_ident> for #ident {
                        type Error = ::gears::error::ProtobufError;

                        fn try_from(value: #raw_ident) -> ::std::result::Result<Self, Self::Error> {
                            ::std::result::Result::Ok(Self {
                                #(#try_from_fields_iter_gen),*
                            })
                        }
                    }

                };

                quote! {
                    #try_from

                    #from_impl

                    #protobuf_trait_impl
                }
            };

            let gen = quote! {
                #gen_struct

                #get_from
            };

            Ok(gen.into())
        }
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Protobuf can be derived only for `struct`",
        )),
    }
}
