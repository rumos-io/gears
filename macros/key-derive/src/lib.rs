#![cfg(not(doctest))]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/","Readme.md"))]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod params_key;
mod store_key;

#[proc_macro_derive(Keys, attributes(proto))]
pub fn proto_derive(input: TokenStream) -> TokenStream {
    expand::expand(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

mod expand {
    use darling::{util::Flag, FromAttributes, FromDeriveInput, FromMeta};
    use proc_macro2::TokenStream;
    use syn::{spanned::Spanned, DeriveInput, Ident};

    use crate::params_key;

    pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
        let KeysArg { kind, gears} = KeysArg::from_derive_input(&input)?;

        match kind.kind(&input.ident)? {
            Kind::Store => todo!(),
            Kind::Params => params_key::expand_params(input, gears),
        }
    }

    #[derive(FromDeriveInput)]
    #[darling(attributes(keys))]
    pub struct KeysArg {
        #[darling(default)]
        pub kind: StoreOrParams,
        #[darling(default)]
        pub gears: Flag,
    }

    #[derive(FromAttributes, Default)]
    #[darling(default, attributes(keys), forward_attrs(allow, doc, cfg))]
    #[darling(and_then = Self::not_empty)]
    pub struct KeysAttr {
        pub to_string: String,
    }

    impl KeysAttr {
        fn not_empty(self) -> darling::Result<Self> {
            if self.to_string.is_empty() {
                Err(darling::Error::custom("key can't be empty").with_span(&self.to_string.span()))
            } else {
                Ok(self)
            }
        }
    }

    #[derive(FromMeta, Default)]
    #[darling(and_then = Self::not_both)]
    pub struct StoreOrParams {
        store: Flag,
        params: Flag,
    }

    pub enum Kind {
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

        pub fn kind(&self, ident: &Ident) -> syn::Result<Kind> {
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
}
