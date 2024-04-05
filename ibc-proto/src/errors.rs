#[derive(thiserror::Error, Debug, PartialEq, Eq)]
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
