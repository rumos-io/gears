use proto_messages::cosmos::ibc::types::core::{
    client::{context::types::Status, error::ClientError},
    host::{
        error::IdentifierError,
        identifiers::{ClientId, ClientType},
    },
};

use crate::params::ParamsError;

#[derive(Debug, thiserror::Error)]
pub enum ModuleErrors {
    #[error("Error while creating client: {0}")]
    ClientCreateError(#[from] ClientCreateError),
    #[error("Error while updating client: {0}")]
    ClientUpdateError(#[from] ClientUpdateError),
}

#[derive(Debug, thiserror::Error)]
pub enum ClientUpdateError {
    #[error("{0}")]
    ClientError(#[from] ClientError),
    #[error("cannot update client {client_id} with status {status}")]
    NotActive { client_id: ClientId, status: Status },
    #[error("SearchError: {0}")]
    SearchError(#[from] SearchError),
    #[error("Unexpected error: {0}")]
    CustomError(String),
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
    #[error("SearchError: {0}")]
    SearchError(#[from] SearchError),
    #[error("Unexpected error: {0}")]
    CustomError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("not found")]
    NotFound,
    #[error("Decode error: {0}")]
    DecodeError(String),
}
