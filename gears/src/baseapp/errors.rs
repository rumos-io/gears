use crate::{application::handlers::node::TxError, types::gas::GasMeteringErrors};

// We start at u16::MAX + 1 to ensure that the error codes don't collide with the Application codes
const OUT_OF_GAS_CODE: u32 = u16::MAX as u32 + 1;
const INVALID_TRANSACTION_CODE: u32 = u16::MAX as u32 + 2;
const INVALID_MESSAGE_CODE: u32 = u16::MAX as u32 + 3;
const GAS_ERRORS_CODE: u32 = u16::MAX as u32 + 4;

#[derive(Debug, Clone, thiserror::Error)]
pub enum RunTxError {
    #[error("there is no block gas left to run the transaction")]
    OutOfBlockGas,
    #[error("{0}")]
    InvalidTransaction(String),
    #[error("{0}")]
    InvalidMessage(String),
    #[error("{0}")]
    GasErrors(#[from] GasMeteringErrors),
    #[error(transparent)]
    Application(#[from] TxError),
}

impl RunTxError {
    pub fn code(&self) -> u32 {
        match self {
            RunTxError::OutOfBlockGas => OUT_OF_GAS_CODE,
            RunTxError::InvalidTransaction(_) => INVALID_TRANSACTION_CODE,
            RunTxError::InvalidMessage(_) => INVALID_MESSAGE_CODE,
            RunTxError::GasErrors(_) => GAS_ERRORS_CODE,
            RunTxError::Application(e) => e.code.value() as u32,
        }
    }

    pub fn codespace(&self) -> &'static str {
        match self {
            RunTxError::Application(e) => e.codespace,
            _ => "base",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error(transparent)]
    Store(#[from] kv_store::error::KVStoreError),
}
