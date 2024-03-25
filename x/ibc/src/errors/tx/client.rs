use gears::error::SearchError;
use proto_messages::cosmos::ibc::types::core::{
    client::{
        context::types::{Height, Status},
        error::ClientError,
    },
    host::{
        error::IdentifierError,
        identifiers::{ClientId, ClientType},
    },
};

use crate::params::ParamsError;

#[derive(Debug, thiserror::Error)]
pub enum ClientErrors {
    #[error("Error while creating client: {0}")]
    Create(#[from] ClientCreateError),
    #[error("Error while updating client: {0}")]
    Update(#[from] ClientUpdateError),
    #[error("Error while upgrading client: {0}")]
    Upgrade(#[from] ClientUpgradeError),
    #[error("Error while recovering client: {0}")]
    Recover(#[from] ClientRecoverError),
    #[error("Unexpected error: {0}")]
    Custom(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ClientRecoverError {
    #[error(
        "subject client state latest height is greater or equal to substitute client state latest height ({subject} >= {substitute})"
    )]
    InvalidHeight { subject: Height, substitute: Height },
    #[error("cannot recover client {client_id} with status {status}")]
    SubjectStatus { client_id: ClientId, status: Status },
    #[error("cannot recover client {client_id} with status {status}")]
    SubstituteStatus { client_id: ClientId, status: Status },
    #[error("{0}")]
    ClientError(#[from] ClientError),
    #[error("SearchError: {0}")]
    SearchError(#[from] SearchError),
}

#[derive(Debug, thiserror::Error)]
pub enum ClientUpgradeError {
    #[error(
        "upgraded client height {upgraded} must be at greater than current client height {current}"
    )]
    HeightError { upgraded: Height, current: Height },
    #[error("cannot upgrade client {client_id} with status {status}")]
    NotActive { client_id: ClientId, status: Status },
    #[error("{0}")]
    ClientError(#[from] ClientError),
    #[error("SearchError: {0}")]
    SearchError(#[from] SearchError),
}

#[derive(Debug, thiserror::Error)]
pub enum ClientUpdateError {
    #[error("{0}")]
    ClientError(#[from] ClientError),
    #[error("cannot update client {client_id} with status {status}")]
    NotActive { client_id: ClientId, status: Status },
    #[error("SearchError: {0}")]
    SearchError(#[from] SearchError),
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
