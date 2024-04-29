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
