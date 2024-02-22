use proto_messages::cosmos::ibc::types::core::{
    client::error::ClientError,
    host::{error::IdentifierError, identifiers::ClientType},
};

use crate::params::ParamsError;

#[derive(Debug, thiserror::Error)]
pub enum ModuleErrors {
    #[error("Error while creating client: {0}")]
    ClientCreateError(#[from] ClientCreateError),
}

#[derive(Debug, thiserror::Error)]
pub enum ClientCreateError {
    #[error("cannot create client of type: {0}")]
    InvalidType(ClientType),
    #[error("client state type {0} is not registered in the allowlist")]
    NotAllowed(ClientType),
    #[error("{0}")]
    ParamsError(#[from] ParamsError),
    #[error("Decode error: {0}")]
    DecodeError(#[from] prost::DecodeError),
    #[error("{0}")]
    IdentifierError(#[from] IdentifierError),
    #[error("{0}")]
    ClientError(#[from] ClientError),
    #[error("Unexpected error: {0}")]
    CustomError(String),
}
