//! Default formatting implementation for address

use crate::types::address::AccAddress;
use crate::types::rendering::screen::Content;

use crate::signing::renderer::value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer};

impl PrimitiveValueRenderer<AccAddress> for DefaultPrimitiveRenderer {
    fn format(value: AccAddress) -> Content {
        Content::try_new(value).expect("addresses cannot be empty")
    }
}
