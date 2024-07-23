use cosmwasm_std::Decimal256RangeExceeded;

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
