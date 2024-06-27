use crate::types::store::gas::errors::GasStoreErrors;

#[derive(Debug, thiserror::Error)]
pub enum SubmissionError {
    #[error("Error processing proposal: {0}")]
    Gas(#[from] GasStoreErrors),
    #[error("Error processing proposal: {0}")]
    Any(#[from] anyhow::Error),
}
