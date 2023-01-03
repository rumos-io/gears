use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum IAVLError {
    RotateError,
}

#[derive(Debug, PartialEq)]
pub enum AppError {
    Bech32(bech32::Error),
    InvalidAddress(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            AppError::Bech32(err) => err.fmt(f),
            AppError::InvalidAddress(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<bech32::Error> for AppError {
    fn from(err: bech32::Error) -> AppError {
        AppError::Bech32(err)
    }
}
