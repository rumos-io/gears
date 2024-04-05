pub use cosmwasm_std::CoinFromStrError;

#[derive(Debug, Clone, thiserror::Error)]
pub enum CoinsError {
    #[error("list of coins is empty")]
    EmptyList,
    #[error("coin amount must be positive")]
    InvalidAmount,
    #[error("coins are not sorted and/or contain duplicates")]
    DuplicatesOrUnsorted,
}