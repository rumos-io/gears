use crate::types::gas::GasMeteringErrors;

#[derive(Debug, thiserror::Error)]
pub enum RunTxError {
    #[error("there is no block gas left to run the transaction, try resubmitting")]
    OutOfGas,
    #[error("the transaction is invalid, {0}")]
    TxParseError(String),
    #[error("the transaction contains an invalid message, {0}")]
    Validation(String),
    #[error("{0}")]
    GasErrors(#[from] GasMeteringErrors),
    #[error(transparent)]
    Application(#[from] anyhow::Error),
}

impl RunTxError {
    pub fn code(&self) -> u32 {
        1
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error(transparent)]
    Store(#[from] kv_store::error::KVStoreError),
}
