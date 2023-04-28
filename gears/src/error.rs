use std::fmt::{Display, Formatter, Result};

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
    Tree(trees::Error),
    IBC(String),
    Genesis(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> Result {
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
            AppError::Tree(err) => err.fmt(f),
            AppError::IBC(msg) => write!(f, "ibc routing error: {}", msg),
            AppError::Genesis(msg) => write!(f, "{}", msg),
        }
    }
}

impl AppError {
    pub fn code(&self) -> u32 {
        return 1;
    }
}

impl std::error::Error for AppError {}

impl From<bech32::Error> for AppError {
    fn from(err: bech32::Error) -> AppError {
        AppError::Bech32(err)
    }
}

impl From<trees::Error> for AppError {
    fn from(err: trees::Error) -> AppError {
        AppError::Tree(err)
    }
}
