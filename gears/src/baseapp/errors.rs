use crate::types::gas::GasErrors;

#[derive(Debug, Clone, thiserror::Error)]
pub enum RunTxError {
    #[error("no block gas left to run tx")]
    OutOfGas,
    #[error("invalid transaction: {0}")]
    TxParseError(String),
    #[error("Message validation error: {0}")]
    Validation(String),
    #[error("Gas errors: {0}")]
    GasErrors( #[from] GasErrors),
    #[error("Custom error: {0}")]
    Custom(String),
}

impl RunTxError {
    pub fn code(&self) -> u32 {
        1
    }
}
