//! This implementation simply returns String without any changes and made for convenient usage as for other types

#[doc(inline)]
use crate::signing::renderer::value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer};

// This is unnecessary but let's keep things convenient

// TODO: Will it be good idea to try_parse to some types like `i64`?

impl PrimitiveValueRenderer<&str> for DefaultPrimitiveRenderer {
    fn format(value: &str) -> String {
        value.to_string()
    }
}

impl PrimitiveValueRenderer<String> for DefaultPrimitiveRenderer {
    fn format(value: String) -> String {
        value
    }
}
