use darling::{util::PathList, FromAttributes};
use quote::quote;
use syn::{DataStruct, DeriveInput, Field, TypePath};

#[derive(FromAttributes)]
#[darling(attributes(proto), forward_attrs(allow, doc, cfg))]
struct ProtobufAttr {
    raw: Option<syn::Path>,
    raw_attributes: PathList,
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
                    raw_attributes,
                } = ProtobufAttr::from_attributes(&attrs)?;
                let raw = raw
                    .map(|path| syn::Type::Path(TypePath { qself: None, path }))
                    .unwrap_or(ty);

                result_fields.push(quote! {
                    #(#raw_attributes,)*
                    #vis #ident : Option<#raw>
                });
            }

            let new_name = syn::Ident::new(
                &format!("Raw{}", ident.to_string()),
                proc_macro2::Span::call_site(),
            );
            let gen = quote! {

                #[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::prost::Message)]
                #(#raw_derives,)*
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
