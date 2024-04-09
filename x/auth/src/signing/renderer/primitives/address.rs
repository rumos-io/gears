//! Default formatting implementation for address

// use proto_messages::cosmos::tx::v1beta1::screen::Content;
// use proto_types::AccAddress;

use gears::ibc::address::AccAddress;
use gears::types::rendering::screen::Content;

use crate::signing::renderer::value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer};

impl PrimitiveValueRenderer<AccAddress> for DefaultPrimitiveRenderer {
    fn format(value: AccAddress) -> Content {
        Content::new(value).expect("addresses cannot be empty")
    }
}
