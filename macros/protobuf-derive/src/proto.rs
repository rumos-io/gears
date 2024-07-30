use darling::{util::Flag, FromAttributes, FromDeriveInput};
use quote::quote;
use syn::{DataStruct, DeriveInput};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(proto))]
struct ProtobufArg {
    #[darling(default)]
    raw: Option<syn::TypePath>,
}

#[derive(FromAttributes, Default)]
#[darling(default, attributes(proto), forward_attrs(allow, doc, cfg))]
struct ProtobufAttr {
    name: Option<syn::Ident>,
    optional: Flag,
}

pub fn expand_raw_existing(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ProtobufArg { raw } = ProtobufArg::from_derive_input(&input)?;
    let DeriveInput { ident, data, .. } = input;

    let raw = match raw {
        Some(raw) => quote! { #raw },
        None => {
            let new_name = syn::Ident::new(
                &format!("Raw{}", ident.to_string()),
                proc_macro2::Span::call_site(),
            );

            quote! { #new_name }
        }
    };

    let protobuf_trait_impl = quote! {
        impl ::gears::tendermint::types::proto::Protobuf<#raw> for #ident {}
    };

    match data {
        syn::Data::Struct(DataStruct { fields, .. }) => {
            let mut raw_fields = Vec::new();
            for field in fields {
                let ProtobufAttr { name, optional } = ProtobufAttr::from_attributes(&field.attrs)?;
                let field_indent = field.ident.ok_or(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "Can't derive on tuple structures",
                ))?;

                let name = name.clone().unwrap_or(field_indent.clone());

                raw_fields.push((name, field_indent, field.ty, optional.is_present()))
            }

            let from_fields_iter_gen = {
                let mut from_fields = Vec::with_capacity(raw_fields.len());

                for (other_name, field_ident, field_type, is_optional) in &raw_fields {
                    let result = match (is_option(&field_type), is_optional) {
                        (true, true) => quote! {
                            #other_name : match value.#field_ident
                            {
                                ::std::option::Option::Some(var) => ::std::option::Option::Some( ::std::convert::Into::into(var)),
                                ::std::option::Option::None => ::std::option::Option::None,
                            }
                        },
                        (true, false) => Err(syn::Error::new(
                            proc_macro2::Span::call_site(),
                            "Can't have optional raw while raw is required",
                        ))?,
                        (false, true) => quote! {
                            #other_name : ::std::option::Option::Some(::std::convert::Into::into(value.#field_ident))
                        },
                        (false, false) => quote! {
                            #other_name : ::std::convert::Into::into(value.#field_ident)
                        },
                    };

                    from_fields.push(result);
                }

                from_fields
            };

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

            let try_from_fields_iter_gen =
                raw_fields
                    .iter()
                    .map(|(other_name, field_ident, field_type, is_optional)| {
                        match (is_option(&field_type), is_optional) {
                            (true, true) => quote! {
                                #field_ident : match value.#other_name {
                                    Some(var) => Some(::std::convert::TryFrom::try_from(var)?),
                                    None => None,
                                }
                            },
                            (true, false) => quote! {
                                #field_ident : ::std::option::Option::Some(::std::convert::Into::into(value.#other_name))
                            },
                            (false, true) => quote! {
                                #field_ident : match value.#other_name
                                {
                                    ::std::option::Option::Some(var) => ::std::result::Result::Ok( ::std::convert::Into::into(var)),
                                    ::std::option::Option::None => ::std::result::Result::Err( ::gears::error::ProtobufError::MissingField( ::std::format!( "Missing field: {}", #other_name ))),
                                }?
                            },
                            (false, false) => quote! {
                                #field_ident : ::std::convert::TryFrom::try_from(value.#other_name)?
                            },
                        }
                    });

            let try_from = quote! {

                impl TryFrom<#raw> for #ident {
                    type Error = ::gears::error::ProtobufError;

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

fn is_option(ty: &syn::Type) -> bool {
    let opt = match ty {
        syn::Type::Path(typepath) if typepath.qself.is_none() => Some(typepath.path.clone()),
        _ => None,
    };

    if let Some(o) = opt {
        check_for_option(&o).is_some()
    } else {
        false
    }
}

fn check_for_option(path: &syn::Path) -> Option<&syn::PathSegment> {
    let idents_of_path = path.segments.iter().fold(String::new(), |mut acc, v| {
        acc.push_str(&v.ident.to_string());
        acc.push(':');
        acc
    });
    vec!["Option:", "std:option:Option:", "core:option:Option:"]
        .into_iter()
        .find(|s| idents_of_path == *s)
        .and_then(|_| path.segments.last())
}
