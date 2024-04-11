#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    #[error(transparent)]
    Decode(#[from] bech32::Error),

    #[error("address has wrong prefix (expected {expected:?}, found {found:?})")]
    InvalidPrefix { expected: String, found: String },

    #[error("invalid variant (expected {expected:?}, found {found:?})")]
    InvalidVariant { expected: String, found: String },

    #[error("invalid length, max length is: {max:?}, found {found:?})")]
    InvalidLength { max: u8, found: usize },

    #[error("bech32 decode error: address is empty")]
    EmptyAddress,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid coins: `{0}`")]
    Coins(String),
    #[error("invalid coin: `{0}`")]
    Coin(String),
    #[error(transparent)]
    Decode(#[from] prost::DecodeError),
    #[error("{0}")]
    DecodeProtobuf(String),
    #[error("decode error: `{0}`")]
    DecodeAny(String),
    #[error("missing field: `{0}`")]
    MissingField(String),
    #[error("decode error: `{0}`")]
    DecodeAddress(String),
    #[error("decode error: `{0}`")]
    DecodeGeneral(String),
    #[error("serde error: {0}")]
    SerdeSerialize(String),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    Custom(String),
    #[error("invalid sign mode: `{0}`")]
    InvalidSignMode(i32),
}
