use darling::{util::PathList, FromAttributes};
use quote::quote;
use syn::{DataStruct, DeriveInput, Field, TypePath};

#[derive(FromAttributes, Default)]
#[darling(default, attributes(proto))]
struct ProtobufAttr {
    raw: Option<syn::Path>,
    attr: PathList,
}

pub fn extend_new_structure(
    DeriveInput {
        ident, data, vis, ..
    }: DeriveInput,
    raw_derives: PathList,
) -> syn::Result<proc_macro2::TokenStream> {
    match data {
        syn::Data::Struct(DataStruct { fields, .. }) => {
            let mut result_fields = Vec::with_capacity(fields.len());
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
                    attr: raw_attributes,
                } = ProtobufAttr::from_attributes(&attrs)?;
                let raw = raw
                    .map(|path| syn::Type::Path(TypePath { qself: None, path }))
                    .unwrap_or(ty);

                result_fields.push(match raw_attributes.is_empty() {
                    true => quote! {
                        #vis #ident : #raw
                    },
                    false => quote! {
                        #[#(#raw_attributes), *]
                        #vis #ident : #raw
                    },
                });
            }

            let new_name = syn::Ident::new(
                &format!("Raw{}", ident.to_string()),
                proc_macro2::Span::call_site(),
            ); // ::std::clone::Clone, ::std::cmp::PartialEq,
            let gen = quote! {

                #[derive(::prost::Message)]
                #[derive(#(#raw_derives,)*)]
                #vis struct  #new_name
                {
                    #(#result_fields),*
                }
            };

            Ok(gen.into())
        }
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Protobuf can be derived only for `struct`",
        )),
    }
}
