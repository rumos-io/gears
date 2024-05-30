use std::fmt::{Display, Formatter, Result};

use kv_store::error::StoreError;

use crate::types::store::errors::StoreErrors;

pub const IBC_ENCODE_UNWRAP: &str = "Should be okay. In future versions of IBC they removed Result";
pub const POISONED_LOCK: &str = "poisoned lock";

#[derive(Debug, PartialEq)]
pub enum AppError {
    Bech32(bech32::Error),
    InvalidRequest(String),
    Send(String),
    AccountNotFound,
    TxParseError(String),
    Coins(String),
    TxValidation(String),
    Timeout { timeout: u64, current: u64 },
    Memo(u64),
    InvalidPublicKey,
    Store(StoreError),
    IBC(String),
    Genesis(String),
    Query(String),
    Custom(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            AppError::Bech32(err) => err.fmt(f),
            AppError::InvalidRequest(msg) => write!(f, "invalid request: {}", msg),
            AppError::Send(msg) => write!(f, "send error: {}", msg),
            AppError::AccountNotFound => write!(f, "account does not exist"),
            AppError::TxParseError(msg) => write!(f, "tx parse error: {}", msg),
            AppError::Coins(msg) => write!(f, "invalid coins: {}", msg),
            AppError::TxValidation(msg) => write!(f, "invalid transaction: {}", msg),
            AppError::Timeout { timeout, current } => write!(
                f,
                "tx has timed out; timeout height: {}, current height: {}",
                timeout, current
            ),
            AppError::Memo(length) => write!(f, "memo is too long, max length is {}", length),
            AppError::InvalidPublicKey => write!(f, "public key is invalid"),
            // AppError::Tree(err) => err.fmt(f),
            AppError::IBC(msg) => write!(f, "ibc routing error: {}", msg),
            AppError::Genesis(msg) => write!(f, "{}", msg),
            AppError::Query(msg) => write!(f, "Error executing query: {msg}"),
            AppError::Store(msg) => write!(f, "Store error: {msg}"),
            AppError::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl AppError {
    pub fn code(&self) -> u32 {
        1
    }
}

impl std::error::Error for AppError {}

impl From<bech32::Error> for AppError {
    fn from(err: bech32::Error) -> AppError {
        AppError::Bech32(err)
    }
}

impl From<StoreError> for AppError {
    fn from(err: StoreError) -> AppError {
        AppError::Store(err)
    }
}

impl From<core_types::errors::Error> for AppError {
    fn from(value: core_types::errors::Error) -> Self {
        Self::IBC(value.to_string())
    }
}

impl From<StoreErrors> for AppError {
    fn from(value: StoreErrors) -> Self {
        Self::Custom(value.to_string())
    }
}
