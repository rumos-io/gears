use super::gas::errors::GasStoreErrors;

#[derive(Debug, Clone, thiserror::Error)]
pub enum StoreErrors {
    #[error("gas error: {0}")]
    Gas(#[from] GasStoreErrors),
}
