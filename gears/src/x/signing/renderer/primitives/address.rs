//! Default formatting implementation for address

use crate::ibc::address::AccAddress;
use crate::types::rendering::screen::Content;

use crate::x::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, PrimitiveValueRenderer,
};

impl PrimitiveValueRenderer<AccAddress> for DefaultPrimitiveRenderer {
    fn format(value: AccAddress) -> Content {
        Content::new(value).expect("addresses cannot be empty")
    }
}
