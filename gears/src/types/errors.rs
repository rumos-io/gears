pub use cosmwasm_std::CoinFromStrError;
pub use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("invalid denom")]
    InvalidDenom,
}
