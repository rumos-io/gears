//! Default formatting implementation for bool

use crate::types::rendering::screen::Content;

use crate::signing::renderer::value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer};

impl PrimitiveValueRenderer<bool> for DefaultPrimitiveRenderer {
    fn format(value: bool) -> Content {
        if value {
            Content::new("True").expect("hard coded String is not empty")
        } else {
            Content::new("False").expect("hard coded String is not empty")
        }
    }
}
