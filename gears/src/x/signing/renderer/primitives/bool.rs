//! Default formatting implementation for bool

// use proto_messages::cosmos::tx::v1beta1::screen::Content;

use crate::types::rendering::screen::Content;

use crate::x::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, PrimitiveValueRenderer,
};

impl PrimitiveValueRenderer<bool> for DefaultPrimitiveRenderer {
    fn format(value: bool) -> Content {
        if value {
            Content::new("True").expect("hard coded String is not empty")
        } else {
            Content::new("False").expect("hard coded String is not empty")
        }
    }
}
