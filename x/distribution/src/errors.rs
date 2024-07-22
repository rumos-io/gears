use gears::{
    error::NumericError,
    types::base::errors::CoinsError,
    x::errors::{AccountNotFound, BankKeeperError},
};

#[derive(Debug, Clone, thiserror::Error)]
pub enum TokenAllocationError {
    #[error(transparent)]
    AccountNotFound(#[from] AccountNotFound),
    #[error("Stored fee pool should not have been none")]
    FeePoolNone,
    #[error("{0}")]
    Numeric(#[from] NumericError),
    #[error("{0}")]
    Coins(#[from] CoinsError),
    #[error("{0}")]
    BankSend(#[from] BankKeeperError),
}
