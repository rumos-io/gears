use crate::signing::renderer::value_renderer::RenderError;
#[doc(inline)]
use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, TryPrimitiveValueRenderer,
};
use crate::types::rendering::screen::Content;

impl TryPrimitiveValueRenderer<&str> for DefaultPrimitiveRenderer {
    fn try_format(value: &str) -> Result<Content, RenderError> {
        if value.is_empty() {
            Err(RenderError::Rendering(
                "cannot render empty string".to_string(),
            ))
        } else {
            Ok(Content::new(value).expect("slice is not empty"))
        }
    }
}

impl TryPrimitiveValueRenderer<String> for DefaultPrimitiveRenderer {
    fn try_format(value: String) -> Result<Content, RenderError> {
        if value.is_empty() {
            Err(RenderError::Rendering(
                "cannot render empty string".to_string(),
            ))
        } else {
            Ok(Content::new(value).expect("String is not empty"))
        }
    }
}
