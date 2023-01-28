use std::fmt::{Display, Formatter, Result};

use prost::DecodeError;
use structopt::clap::App;

#[derive(Debug)]
pub enum IAVLError {
    RotateError,
}

#[derive(Debug, PartialEq)]
pub enum AppError {
    Bech32(bech32::Error),
    Prost(DecodeError),
    InvalidRequest(String),
    Send(String),
    AccountNotFound,
    TxParseError(String),
    Coins(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            AppError::Bech32(err) => err.fmt(f),
            AppError::InvalidRequest(msg) => write!(f, "invalid request: {}", msg),
            AppError::Send(msg) => write!(f, "send error: {}", msg),
            AppError::AccountNotFound => write!(f, "account does not exist"),
            AppError::Prost(err) => err.fmt(f),
            AppError::TxParseError(msg) => write!(f, "tx parse error: {}", msg),
            AppError::Coins(msg) => write!(f, "invalid coins: {}", msg),
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

impl From<DecodeError> for AppError {
    fn from(err: DecodeError) -> AppError {
        AppError::Prost(err)
    }
}
