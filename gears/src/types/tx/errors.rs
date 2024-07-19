use crate::types::auth::info::AuthError;

#[derive(Debug, thiserror::Error)]
#[error("transaction applies non empty set of messages")]
pub struct EmptyMessagesError;

#[derive(Debug, thiserror::Error)]
pub enum TxError {
    #[error("{0}")]
    Decode(#[from] prost::DecodeError),
    #[error("decode error: `{0}`")]
    DecodeGeneral(String),
    #[error("missing field: `{0}`")]
    MissingField(String),
    #[error("{0}")]
    Auth(#[from] AuthError),
    #[error("{0}")]
    Ibc(#[from] core_types::errors::CoreError),
}
