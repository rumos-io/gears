use thiserror::Error;

use crate::baseapp::errors::TxValidationError;

#[derive(Debug, Error, PartialEq)]
pub enum AppError {
    #[error("{0}")]
    Bech32(#[from] bech32::Error),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("send error: {0}")]
    Send(String),
    #[error("account does not exist")]
    AccountNotFound,
    #[error("tx parse error: {0}")]
    TxParseError(String),
    #[error("invalid coins: {0}")]
    Coins(String),
    #[error("invalid transaction: {0}")]
    TxValidation(#[from] TxValidationError),
    #[error("memo is too long, max length is {0}")]
    Memo(u64),
    #[error("public key is invalid")]
    InvalidPublicKey,
    #[error("{0}")]
    Tree(trees::Error),
    #[error("ibc routing error: {0}")]
    IBC(String),
    #[error("{0}")]
    Genesis(String),
}

impl AppError {
    pub fn code(&self) -> u32 {
        return 1;
    }
}

impl From<trees::Error> for AppError {
    fn from(err: trees::Error) -> AppError {
        AppError::Tree(err)
    }
}

impl From<ibc_proto::protobuf::Error> for AppError {
    fn from(err: ibc_proto::protobuf::Error) -> AppError {
        AppError::InvalidRequest(err.to_string())
    }
}
