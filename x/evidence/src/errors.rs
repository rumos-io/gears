use gears::tendermint::informal::Hash;

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
