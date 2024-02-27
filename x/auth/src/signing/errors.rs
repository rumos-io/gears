// use super::types::{screen::{ContentError, IndentError}, signer_data::ChainIdError};

#[derive(Debug, thiserror::Error)]
pub enum SigningErrors {
    #[error("{0}")]
    CustomError(String),
    #[error("EncodeError")]
    EncodeError,
    #[error("{0}")]
    IoError(#[from] std::io::Error),
}
