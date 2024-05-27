#[derive(Debug, Clone, thiserror::Error)]
pub enum SendCoinsError {
    #[error("list of coins is empty")]
    EmptyList,
    #[error("coin amount must be positive")]
    InvalidAmount,
    #[error("coins are not sorted and/or contain duplicates")]
    DuplicatesOrUnsorted,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum CoinsError {
    #[error("Denom parse error: {0}")]
    Denom(String),
    #[error("Uint256 parse error: {0}")]
    Uint(String),
    #[error("Decimal256 parse error: {0}")]
    Decimal(String),
}

#[derive(Debug, thiserror::Error)]
pub enum CoinsParseError {
    #[error("Failed to parse: {0}")]
    Parse(#[from] CoinsError),
    #[error("Parsed invalid coins: {0}")]
    Validate(#[from] SendCoinsError),
}
