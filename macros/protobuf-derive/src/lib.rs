use darling::{util::Flag, FromMeta};
use proc_macro::TokenStream;
use syn::{parse_macro_input, spanned::Spanned};

mod proto;
mod raw;

#[proc_macro_derive(Protobuf, attributes(proto))]
pub fn proto_derive(input: TokenStream) -> TokenStream {
    proto::expand_raw_existing(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Raw, attributes(raw))]
pub fn raw_derive(input: TokenStream) -> TokenStream {
    raw::extend_new_structure(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[derive(FromMeta, Default)]
#[darling(and_then = Self::not_both)]
struct OptionalOrRepeated {
    optional: Flag,
    repeated: Flag,
}

impl OptionalOrRepeated {
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

    pub fn kind(self) -> FieldWrapper {
        match (self.optional.is_present(), self.repeated.is_present()) {
            (true, true) => unreachable!("we validated struct so its not possible"),
            (true, false) => FieldWrapper::Optional,
            (false, true) => FieldWrapper::Vec,
            (false, false) => FieldWrapper::None,
        }
    }
}

enum FieldWrapper {
    Optional,
    Vec,
    None,
}

impl FieldWrapper {
    pub fn from_type(ty: &syn::Type) -> syn::Result<FieldWrapper> {
        match (Self::is_option(ty), Self::is_vec(ty)) {
            (true, true) => Err(syn::Error::new(
                ty.span(),
                "Can't use field with `Option` and `Vec`",
            )),
            (true, false) => Ok(FieldWrapper::Optional),
            (false, true) => Ok(FieldWrapper::Vec),
            (false, false) => Ok(FieldWrapper::None),
        }
    }

    fn is_option(ty: &syn::Type) -> bool {
        let opt = match ty {
            syn::Type::Path(typepath) if typepath.qself.is_none() => Some(typepath.path.clone()),
            _ => None,
        };

        if let Some(o) = opt {
            Self::check_for_option(&o).is_some()
        } else {
            false
        }
    }

    fn is_vec(ty: &syn::Type) -> bool {
        let opt = match ty {
            syn::Type::Path(typepath) if typepath.qself.is_none() => Some(typepath.path.clone()),
            _ => None,
        };

        if let Some(o) = opt {
            Self::check_for_vec(&o).is_some()
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
        vec![
            "Option:",
            "std:option:Option:",
            "core:option:Option:",
            "option:Option:",
        ]
        .into_iter()
        .find(|s| idents_of_path == *s)
        .and_then(|_| path.segments.last())
    }

    fn check_for_vec(path: &syn::Path) -> Option<&syn::PathSegment> {
        let idents_of_path = path.segments.iter().fold(String::new(), |mut acc, v| {
            acc.push_str(&v.ident.to_string());
            acc.push(':');
            acc
        });
        vec!["Vec:", "std:vec:Vec:", "vec:Vec:"]
            .into_iter()
            .find(|s| idents_of_path == *s)
            .and_then(|_| path.segments.last())
    }
}
