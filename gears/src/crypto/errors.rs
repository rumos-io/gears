// use k256::ecdsa;

// use secp256k1::ecdsa;

#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    #[error("{0}")]
    Secp(#[from] secp256k1::Error),
    #[error("{0}")]
    K256(#[from] k256::ecdsa::Error),
}
