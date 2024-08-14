use gears::{tendermint::informal::Hash, types::address::ConsAddress};

#[derive(Debug, thiserror::Error)]
pub enum GenesisStateError {
    #[error("failure in conversion of any type into concrete evidence")]
    Decode,
    #[error("{0}")]
    Validation(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum EvidenceError {
    #[error("Evidence with hash {0} already exists")]
    AlreadyExists(Hash),
}

#[derive(Debug, thiserror::Error)]
pub enum EquivocationEvidenceError {
    #[error("expected signing info for validator {0} but not found")]
    SigningInfoNotExists(ConsAddress),
}
