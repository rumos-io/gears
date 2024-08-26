#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(thiserror::Error, Debug)]
pub enum RenderError {
    #[error("`{0}`")]
    Rendering(String),
}

/// A renderer for amino signature handler.
pub trait AminoRenderer {
    /// Render a message into a compatible amino json struct.
    fn render(&self) -> Result<serde_json::Map<String, serde_json::Value>, RenderError>;
}

impl<T: serde::Serialize> AminoRenderer for T {
    fn render(&self) -> Result<serde_json::Map<String, serde_json::Value>, RenderError> {
        let mut value: serde_json::Map<String, serde_json::Value> = serde_json::from_slice(
            &serde_json::to_vec(&self).map_err(|e| RenderError::Rendering(e.to_string()))?,
        )
        .map_err(|e| RenderError::Rendering(e.to_string()))?;
        value.remove("@type");

        // fn filter_type(value: &mut serde_json::Map<String, serde_json::Value>) {
        //     value.remove("@type");
        //     // for (_k, v) in value.iter_mut() {
        //     //     if v.is_object() {
        //     //         filter_type(v.as_object_mut().expect("condition is checked"));
        //     //     }
        //     // }
        // }
        //
        // filter_type(&mut value);

        Ok(value)
    }
}
