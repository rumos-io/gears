use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("rotate error")]
    RotateError,
    // #[error("invalid coins: `{0}`")]
    // Coins(String),
    // #[error("invalid coin: `{0}`")]
    // Coin(String),
    // #[error("decode error")]
    // Decode(#[from] prost::DecodeError),
    // #[error("decode error: `{0}`")]
    // DecodeAny(String),
    // #[error("missing field: `{0}`")]
    // MissingField(String),
    // #[error("decode error: `{0}`")]
    // DecodeAddress(String),
    // #[error("decode error: `{0}`")]
    // DecodeGeneral(String),
}
