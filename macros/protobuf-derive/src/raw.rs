use darling::FromAttributes;
use quote::quote;
use syn::{DataStruct, DeriveInput};

#[derive(FromAttributes, Default)]
#[darling(default, attributes(proto), forward_attrs(allow, doc, cfg))]
struct RawProtobufAttr {
    name: Option<syn::Ident>,
}

pub fn expand_raw_existing(
    raw: syn::Type,
    DeriveInput { ident, data, .. }: DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let protobuf_trait_impl = quote! {
        impl ::gears::tendermint::types::proto::Protobuf<#raw> for #ident {}
    };

    match data {
        syn::Data::Struct(DataStruct { fields, .. }) => {
            let mut raw_fields = Vec::new();
            for field in fields {
                let RawProtobufAttr { name } = RawProtobufAttr::from_attributes(&field.attrs)?;

                raw_fields.push((
                    name,
                    field.ident.ok_or(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        "Can't derive on tuple structures",
                    ))?,
                ))
            }

            let from_fields_iter_gen = raw_fields.iter().map(|(other_name, field_ident)| {
                let other_name = other_name.clone().unwrap_or(field_ident.clone());

                quote! {
                    #other_name : ::std::convert::Into::into(value.#field_ident)
                }
            });

            let from_impl = quote! {
                impl ::std::convert::From<#ident> for #raw {
                    fn from(value: #ident) -> Self {
                        Self
                        {
                            #(#from_fields_iter_gen),*
                        }
                    }
                }
            };

            let try_from_fields_iter_gen = raw_fields.iter().map(|(other_name, field_ident)| {
                let other_name = other_name.clone().unwrap_or(field_ident.clone());

                quote! {
                    #field_ident : ::std::convert::TryFrom::try_from(value.#other_name)?
                }
            });

            let try_from = quote! {

                impl TryFrom<#raw> for #ident {
                    type Error = ::gears::core::errors::CoreError;

                    fn try_from(value: #raw) -> ::std::result::Result<Self, Self::Error> {
                        ::std::result::Result::Ok(Self {
                            #(#try_from_fields_iter_gen),*
                        })
                    }
                }

            };

            let gen = quote! {
                #try_from

                #from_impl

                #protobuf_trait_impl
            };

            Ok(gen.into())
        }
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Protobuf can be derived only for `struct`",
        )),
    }
}
