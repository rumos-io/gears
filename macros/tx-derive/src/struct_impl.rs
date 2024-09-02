use syn::{DataStruct, Ident};

pub fn expand_macro(
    DataStruct { struct_token, fields, semi_token  }: DataStruct,
    type_ident: Ident,
    crate_prefix: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> 
{
 todo!()
}