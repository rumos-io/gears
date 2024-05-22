#[derive(Debug, thiserror::Error)]
pub enum ParamsError {
    #[error("params not found")]
    NotFound,
    #[error("missing field of param")]
    MissingField,
    #[error("deserialize error: {0}")]
    Deserialization(String),
}

impl From<serde_json::Error> for ParamsError {
    fn from(value: serde_json::Error) -> Self {
        Self::Deserialization(value.to_string())
    }
}

impl From<prost::DecodeError> for ParamsError {
    fn from(value: prost::DecodeError) -> Self {
        Self::Deserialization(value.to_string())
    }
}
