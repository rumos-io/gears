use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
#[error("Round up operation failed because of overflow")]
pub struct RoundUpOverflowError;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum AddressError {
    #[error("failed to decode")]
    Decode(#[from] bech32::Error),

    #[error("address has wrong prefix (expected {expected:?}, found {found:?})")]
    InvalidPrefix { expected: String, found: String },

    #[error("invalid variant (expected {expected:?}, found {found:?})")]
    InvalidVariant { expected: String, found: String },

    #[error("invalid length, max length is: {max:?}, found {found:?})")]
    InvalidLength { max: u8, found: usize },
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("invalid denom")]
    InvalidDenom,
}
