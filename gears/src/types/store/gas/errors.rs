use crate::types::{auth::gas::GasError, gas::GasMeteringErrors};

use super::ext::UnwrapGasError;

// TODO: this error should have two variants, out of gas and gas overflow
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub enum GasStoreErrorKinds {
    #[error("Metering error: {0}")]
    Metering(#[from] GasMeteringErrors),
    #[error("Gas error: {0}")]
    Gas(#[from] GasError),
}

#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
#[error("gas error: {kind}")]
pub struct GasStoreErrors {
    pub key: Vec<u8>,
    pub kind: GasStoreErrorKinds,
}

impl GasStoreErrors {
    pub fn new(key: &[u8], kind: impl Into<GasStoreErrorKinds>) -> Self {
        Self {
            key: key.to_vec(),
            kind: kind.into(),
        }
    }
}

impl UnwrapGasError for GasStoreErrors {}
