//! Default formating implementation for bool

use crate::signing::renderer::value_renderer::{PrimitiveDefaultRenderer, PrimitiveValueRenderer};

impl PrimitiveValueRenderer<bool> for PrimitiveDefaultRenderer {
    fn format(value: bool) -> String {
        if value {
            "True".to_string()
        } else {
            "False".to_string()
        }
    }
}
