// use proto_messages::cosmos::tx::v1beta1::screen::Content;

use gears::types::rendering::screen::Content;

use crate::signing::renderer::value_renderer::Error;
#[doc(inline)]
use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, TryPrimitiveValueRenderer,
};

impl TryPrimitiveValueRenderer<&str> for DefaultPrimitiveRenderer {
    fn try_format(value: &str) -> Result<Content, Error> {
        if value.is_empty() {
            Err(Error::Rendering("cannot render empty string".to_string()))
        } else {
            Ok(Content::new(value).expect("slice is not empty"))
        }
    }
}

impl TryPrimitiveValueRenderer<String> for DefaultPrimitiveRenderer {
    fn try_format(value: String) -> Result<Content, Error> {
        if value.is_empty() {
            Err(Error::Rendering("cannot render empty string".to_string()))
        } else {
            Ok(Content::new(value).expect("String is not empty"))
        }
    }
}
