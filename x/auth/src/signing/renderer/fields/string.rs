//! This implementation simply returns String without any changes and made for convenient usage as for other types

#[doc(inline)]
use crate::signing::renderer::value_renderer::{PrimitiveDefaultRenderer, PrimitiveValueRenderer};

// This is unnecessary but let's keep things convinient

impl PrimitiveValueRenderer<&str> for PrimitiveDefaultRenderer {
    fn format(value: &str) -> String {
        value.to_string()
    }
}

impl PrimitiveValueRenderer<String> for PrimitiveDefaultRenderer {
    fn format(value: String) -> String {
        value
    }
}
