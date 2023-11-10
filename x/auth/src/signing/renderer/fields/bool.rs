//! Default formating implementation for bool

use crate::signing::renderer::value_renderer::{DefaultRenderer, PrimitiveValueRenderer};

impl PrimitiveValueRenderer<bool> for DefaultRenderer {
    fn format(value: bool) -> String {
        if value {
            "True".to_string()
        } else {
            "False".to_string()
        }
    }
}
