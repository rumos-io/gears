//! Default formatting implementation for bool

use crate::signing::renderer::value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer};

impl PrimitiveValueRenderer<bool> for DefaultPrimitiveRenderer {
    fn format(value: bool) -> String {
        if value {
            "True".to_string()
        } else {
            "False".to_string()
        }
    }
}
