use address::AddressError;
use core_types::errors::CoreError;
use cosmwasm_std::Decimal256RangeExceeded;

use crate::types::{errors::DenomError, tx::metadata::MetadataParseError};

pub const IBC_ENCODE_UNWRAP: &str = "Should be okay. In future versions of IBC they removed Result";
pub const POISONED_LOCK: &str = "poisoned lock";

#[derive(Debug, thiserror::Error)]
pub enum NumericError {
    #[error("overflow on {0}")]
    Overflow(MathOperation),
    #[error("{0}")]
    DecimalRange(#[from] Decimal256RangeExceeded),
}

impl Clone for NumericError {
    fn clone(&self) -> Self {
        match self {
            Self::Overflow(arg0) => Self::Overflow(arg0.clone()),
            Self::DecimalRange(_) => Self::DecimalRange(Decimal256RangeExceeded), // Why ZST is not clonable... Why?
        }
    }
}

#[derive(Debug, Clone, strum::Display)]
pub enum MathOperation {
    Add,
    Sub,
    Div,
    Mul,
}

#[derive(Debug, thiserror::Error)]
pub enum ProtobufError {
    #[error("{0}")]
    MissingField(String),
    #[error("{0}")]
    Metadata(#[from] MetadataParseError),
    #[error("{0}")]
    Denom(#[from] DenomError),
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error("decode adress error: {0}")]
    AddressError(#[from] AddressError),
    #[error("{0}")]
    Custom(#[from] anyhow::Error),
}

impl From<ProtobufError> for tonic::Status {
    fn from(e: ProtobufError) -> Self {
        tonic::Status::invalid_argument(format!("{:?}", e))
    }
}

impl From<std::convert::Infallible> for ProtobufError {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!("who would return infallible error?")
    }
}
