// #![cfg(not(doctest))]
// #![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/","Readme.md"))]

use darling::{util::Flag, FromAttributes, FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Ident};

mod params_key;
mod store_key;

#[proc_macro_derive(Keys, attributes(proto))]
pub fn proto_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    // proto::expand_raw_existing(parse_macro_input!(input))
    //     .unwrap_or_else(syn::Error::into_compile_error)
    //     .into()

    todo!()
}

#[derive(FromDeriveInput)]
#[darling(attributes(keys))]
struct KeysArg {
    #[darling(default)]
    pub kind: StoreOrParams,
    #[darling(default)]
    pub gears: Flag,
}

#[derive(FromAttributes, Default)]
#[darling(default, attributes(keys), forward_attrs(allow, doc, cfg))]
struct KeysAttr {
    name: Option<syn::Ident>,
}

#[derive(FromMeta, Default)]
#[darling(and_then = Self::not_both)]
struct StoreOrParams {
    store: Flag,
    params: Flag,
}

enum Kind {
    Store,
    Params,
}

impl StoreOrParams {
    fn not_both(self) -> darling::Result<Self> {
        if self.store.is_present() && self.params.is_present() {
            Err(darling::Error::custom("Cannot set `store` and `params`")
                .with_span(&self.params.span()))
        } else {
            Ok(self)
        }
    }

    fn kind(&self, ident: &Ident) -> syn::Result<Kind> {
        match (self.params.is_present(), self.params.is_present()) {
            (true, true) => unreachable!("validated it"),
            (true, false) => Ok(Kind::Params),
            (false, true) => Ok(Kind::Store),
            (false, false) => {
                let ident_str = ident.to_string().to_lowercase();

                match (ident_str.contains("storekey"), ident_str.contains("params")) {
                    (true, true) => Ok(Kind::Params),
                    (true, false) => Ok(Kind::Store),
                    _ => Err(syn::Error::new(
                        ident.span(),
                        "requires to set `store` or `params` flag",
                    )),
                }
            }
        }
    }
}
