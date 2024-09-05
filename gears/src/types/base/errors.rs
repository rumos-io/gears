#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoinsError {
    #[error("list of coins is empty")]
    EmptyList,
    #[error("coin amount must be positive")]
    InvalidAmount,
    #[error("coins contain duplicates")]
    Duplicates,
    #[error("coins are not sorted")]
    Unsorted,
    #[error("coin error: {0}")]
    Coin(String),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoinError {
    #[error("Denom parse error: {0}")]
    Denom(String),
    #[error("Uint256 parse error: {0}")]
    Uint(String),
    #[error("Decimal256 parse error: {0}")]
    Decimal(String),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CoinsParseError {
    #[error("Failed to parse: {0}")]
    Parse(#[from] CoinError),
    #[error("Parsed invalid coins: {0}")]
    Validate(#[from] CoinsError),
}
