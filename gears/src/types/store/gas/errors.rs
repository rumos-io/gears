use crate::types::{auth::gas::GasError, gas::GasMeteringErrors};

// TODO: this error should have two variants, out of gas and gas overflow
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub enum GasStoreErrors {
    #[error("Metering error: {0}")]
    Metering(#[from] GasMeteringErrors),
    #[error("Gas error: {0}")]
    Gas(#[from] GasError),
}
