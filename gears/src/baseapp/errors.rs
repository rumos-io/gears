use thiserror::Error;

/// Errors during transaction validation
#[derive(Error, Debug, PartialEq)]
pub enum TxValidationError {
    ///
    #[error("Must contain at least one message")]
    InvalidRequest,
    /// Signature list is empty
    #[error("Signature list is empty")]
    EmptySignList,
    /// Account not found
    #[error("Account not found")]
    AccountNotFound,
    /// Wrong number of signatures
    #[error("wrong number of signatures; expected {expected}, got {got}")]
    WrongSignaturesNum { expected: usize, got: usize },
    /// Wrong number of signer info
    #[error("wrong number of signer info; expected {expected}, got {got}")]
    WrongNumSignerInfo { expected: usize, got: usize },
    /// Incorrect tx sequence
    #[error("incorrect tx sequence; expected {expected}, got {got}")]
    IncorrectSequence { expected: u64, got: u64 },
    /// Secp256 error
    #[error("{0}")]
    Secp256Error(#[from] secp256k1::Error),
    /// Invalid pub key
    #[error("public key is invalid")]
    InvalidPublicKey,
    /// Custom error
    #[error("{0}")]
    CustomError(String),
}

impl TxValidationError {
    pub const fn code(&self) -> u32 {
        1
    }
}

/// Error for timed out cases in tx
#[derive(Error, Debug, PartialEq)]
#[error("tx has timed out; timeout height: {timeout}, current height: {current}")]
pub struct TimeoutError {
    pub timeout: u64,
    pub current: u64,
}

#[derive(Error, Debug, PartialEq)]
pub enum AnteErrors {
    #[error("{0}")]
    CustomError(String),
    #[error("{0}")]
    TxValidation(#[from] TxValidationError),
    #[error("{0}")]
    Timeout(#[from] TimeoutError),
    #[error("memo is too long, max length is {0}")]
    Memo(u64),
}

#[derive(Error, Debug)]

pub enum RunTxError {
    #[error("{0}")]
    CustomError(String),
    #[error("{0}")]
    AnteError(#[from] AnteErrors),
    #[error("tx parse error: {0}")]
    TxParseError(#[from] proto_messages::Error),
    #[error("{0}")]
    TxValidation(#[from] TxValidationError),
    #[error("no block gas left to run tx")]
    OutOfGas,
    #[error("{0}")]
    GasErrors(#[from] crate::types::gas::gas_meter::GasErrors),
}

impl RunTxError {
    pub const fn code(&self) -> u32 {
        1
    }
}
