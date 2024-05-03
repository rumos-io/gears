use gears::error::AppError;
//use gears::error::SearchError;
use ibc::core::{client::types::error::ClientError, host::types::error::IdentifierError};
use prost::DecodeError;
// use proto_messages::cosmos::ibc::types::core::{
//     client::error::ClientError, host::error::IdentifierError,
// };

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("not found")]
    NotFound,
    #[error("Decode error: {0}")]
    DecodeError(String),
}

impl From<prost::DecodeError> for SearchError {
    fn from(value: prost::DecodeError) -> Self {
        Self::DecodeError(value.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientErrors {
    #[error("{0}")]
    Params(#[from] ParamsError),
    #[error("{0}")]
    State(#[from] StateError),
    #[error("{0}")]
    States(#[from] StatesError),
    #[error("{0}")]
    Status(#[from] StatusError),
    #[error("{0}")]
    ConsensusStateHeight(#[from] ConsensusStateHeightError),
    #[error("{0}")]
    ConsensusState(#[from] ConsensusStateError),
    #[error("{0}")]
    ConsensusStates(#[from] ConsensusStatesError),
    #[error("Request decode: {0}")]
    DecodeError(#[from] DecodeError),
    #[error("query path not found")]
    PathNotFound,
}

impl From<ClientErrors> for AppError {
    fn from(e: ClientErrors) -> Self {
        AppError::Query(e.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParamsError {
    #[error("{0}")]
    SearchError(#[from] SearchError),
}

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("{0}")]
    SearchError(#[from] SearchError),
    #[error("Client: {0}")]
    ClientError(#[from] ClientError),
    #[error("Invalid client_id: {0}")]
    IdentifierError(#[from] IdentifierError),
}

#[derive(Debug, thiserror::Error)]
pub enum StatesError {
    #[error("{0}")]
    SearchError(#[from] SearchError),
    #[error("Client: {0}")]
    ClientError(#[from] ClientError),
    #[error("Invalid client_id: {0}")]
    IdentifierError(#[from] IdentifierError),
    #[error("Decode: {0}")]
    DecodeError(#[from] DecodeError),
    #[error("Custom: {0}")]
    Custom(String),
}

#[derive(Debug, thiserror::Error)]
pub enum StatusError {
    #[error("{0}")]
    SearchError(#[from] SearchError),
    #[error("Invalid client_id: {0}")]
    IdentifierError(#[from] IdentifierError),
    #[error("Client: {0}")]
    ClientError(#[from] ClientError),
}

#[derive(Debug, thiserror::Error)]
pub enum ConsensusStateHeightError {
    #[error("Invalid client_id: {0}")]
    IdentifierError(#[from] IdentifierError),
    #[error("Decode: {0}")]
    Decode(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ConsensusStateError {
    #[error("{0}")]
    SearchError(#[from] SearchError),
    #[error("Invalid client_id: {0}")]
    IdentifierError(#[from] IdentifierError),
    #[error("Client: {0}")]
    ClientError(#[from] ClientError),
}

#[derive(Debug, thiserror::Error)]
pub enum ConsensusStatesError {
    #[error("Invalid client_id: {0}")]
    IdentifierError(#[from] IdentifierError),
    #[error("Decode: {0}")]
    DecodeError(#[from] DecodeError),
    #[error("Client: {0}")]
    ClientError(#[from] ClientError),
}
