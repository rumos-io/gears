pub use cosmwasm_std::CoinFromStrError;
pub use cosmwasm_std::StdError;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("invalid denom")]
pub struct DenomError;
