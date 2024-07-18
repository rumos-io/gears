use std::fmt::Display;

use address::AccAddress;
use thiserror::Error;

use crate::{
    application::handlers::node::{ErrorCode, TxError},
    error::AppError,
    types::{
        gas::GasMeteringErrors,
        store::gas::errors::{GasStoreErrorKinds, GasStoreErrors},
    },
};

#[derive(Debug, thiserror::Error)]
pub enum SignVerificationError {
    #[error("signature list is empty")]
    EmptySignatureList,
    #[error("wrong number of signatures; expected {expected}, got {got}")]
    WrongSignatureList { expected: usize, got: usize },
    #[error("account does not exist")]
    AccountNotFound,
    #[error("pubkey on account is not set")]
    PubKeyNotSet,
    #[error("account sequence mismatch, expected {expected}, got {got}")]
    AccountSequence { expected: u64, got: u64 },
}

#[derive(Debug)]
pub(crate) enum AnteGasError {
    Overflow(String),
    OutOfGas(String),
}

impl Display for AnteGasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnteGasError::Overflow(msg) => write!(f, "{msg}"),
            AnteGasError::OutOfGas(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AnteGasError {}

impl From<GasMeteringErrors> for AnteGasError {
    fn from(error: GasMeteringErrors) -> Self {
        match error {
            GasMeteringErrors::ErrorGasOverflow(msg) => AnteGasError::Overflow(msg),
            GasMeteringErrors::ErrorOutOfGas(msg) => AnteGasError::OutOfGas(msg),
        }
    }
}

impl From<GasStoreErrors> for AnteGasError {
    fn from(error: GasStoreErrors) -> Self {
        match error.kind {
            GasStoreErrorKinds::Metering(e) => e.into(),
            GasStoreErrorKinds::Gas(e) => AnteGasError::Overflow(e.to_string()),
        }
    }
}

#[derive(Error, Debug)]
pub(crate) enum AnteError {
    #[error("insufficient fees; got: {got} required: {required}")]
    InsufficientFees { got: String, required: String },
    #[error("fee required")]
    MissingFee,
    #[error("{0}")]
    Validation(String), //TODO: consider breaking this down into more specific errors
    #[error("tx has timed out; timeout height: {timeout}, current height: {current}")]
    Timeout { timeout: u32, current: u32 },
    #[error("{0}")]
    GasError(#[from] AnteGasError),
    #[error("memo is too long, max length is {0}")]
    Memo(u64),
    #[error("tx is too long")]
    TxLen,
    #[error("account not found {0}")]
    AccountNotFound(AccAddress),
    #[error(transparent)]
    Other(#[from] AppError), //TODO: remove this once AppError is removed
}

impl From<AnteError> for TxError {
    fn from(error: AnteError) -> Self {
        let code = match &error {
            AnteError::InsufficientFees {
                got: _,
                required: _,
            } => 1,
            AnteError::MissingFee => 2,

            AnteError::Validation(_) => 3,
            AnteError::Timeout {
                timeout: _,
                current: _,
            } => 4,
            AnteError::GasError(_) => 5,
            AnteError::Memo(_) => 6,
            AnteError::TxLen => 7,
            AnteError::AccountNotFound(_) => 8,
            AnteError::Other(_) => 9,
        };

        TxError {
            msg: format!("{error}"),
            code: ErrorCode::try_new(code).expect("all > 0"),
            codespace: "ante",
        }
    }
}
