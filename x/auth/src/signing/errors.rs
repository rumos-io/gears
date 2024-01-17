// use super::types::{screen::{ContentError, IndentError}, signer_data::ChainIdError};

use ibc_proto::protobuf;

#[derive(Debug, thiserror::Error)]
pub enum SigningErrors {
    #[error("{0}")]
    CustomError(String),
    #[error("EncodeError")]
    EncodeError,
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    ProtoError(#[from] protobuf::Error),
}
