use cosmwasm_std::Decimal256RangeExceeded;

pub const IBC_ENCODE_UNWRAP: &str = "Should be okay. In future versions of IBC they removed Result";
pub const POISONED_LOCK: &str = "poisoned lock";

#[derive(Debug, thiserror::Error)]
pub enum NumericError {
    #[error("TODO")]
    Overflow(MathOperation),
    #[error("TODO")]
    DecimalRange(#[from] Decimal256RangeExceeded),
}

#[derive(Debug, Clone)]
pub enum MathOperation {
    Add,
    Sub,
    Div,
    Mul,
}
