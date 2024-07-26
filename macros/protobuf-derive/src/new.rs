use darling::FromAttributes;
use quote::quote;
use syn::{DataStruct, DeriveInput, Field};

#[derive(FromAttributes, Default)]
#[darling(default, attributes(proto), forward_attrs(allow, doc, cfg))]
struct ProtobufAttr {
    raw: Option<syn::TypePath>,
    kind: String,
    optional: bool,
    repeated: bool,
    tag: Option<u32>,
}

pub fn extend_new_structure(
    DeriveInput {
        ident, data, vis, ..
    }: DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    match data {
        syn::Data::Struct(DataStruct { fields, .. }) => {
            let mut tag_counter = 1;
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
                    optional,
                    repeated,
                    tag,
                    kind,
                } = ProtobufAttr::from_attributes(&attrs)?;
                let raw = raw.map(|this| syn::Type::Path(this)).unwrap_or(ty);
                let tag = tag
                    .inspect(|this| tag_counter = *this)
                    .unwrap_or(tag_counter);

                let result = match (optional, repeated) {
                    (true, true) => Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        "repeated and optional is exclusive",
                    ))?,
                    (true, false) => quote! {
                        #[prost(#kind, optional, tag = #tag.to_string())]
                        #vis #ident : Option<#raw>
                    },
                    (false, true) => quote! {
                        #[prost(#kind, required, repeated, tag = #tag.to_string())]
                        #vis #ident : Vec<#raw>
                    },
                    (false, false) => quote! {
                        #[prost(#kind, required, tag = #tag.to_string())]
                        #vis #ident : #raw>
                    },
                };

                result_fields.push(result);
                tag_counter += 1;
            }

            let new_name = format!("Raw{}", ident.to_string());
            let gen = quote! {

                #[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::prost::Message)]
                #vis struct #new_name
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
