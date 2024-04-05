use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::{
    errors::{CoinFromStrError, CoinsError},
    Coin,
};

// Represents a list of coins with the following properties:
// - Contains at least one coin
// - All coin amounts are positive
// - No duplicate denominations
// - Sorted lexicographically
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct SendCoins(Vec<Coin>);

impl SendCoins {
    pub fn new(coins: Vec<Coin>) -> Result<SendCoins, CoinsError> {
        Self::validate_coins(&coins)?;

        Ok(SendCoins(coins))
    }

    // Checks that the SendCoins are sorted, have positive amount, with a valid and unique
    // denomination (i.e no duplicates). Otherwise, it returns an error.
    // A valid list of coins satisfies:
    // - Contains at least one coin
    // - All amounts are positive
    // - No duplicate denominations
    // - Sorted lexicographically
    // TODO: implement ordering on coins or denominations so that conversion to string can be avoided
    fn validate_coins(coins: &Vec<Coin>) -> Result<(), CoinsError> {
        if coins.is_empty() {
            return Err(CoinsError::EmptyList);
        }

        if coins[0].amount.is_zero() {
            return Err(CoinsError::InvalidAmount);
        };

        let mut previous_denom = coins[0].denom.to_string();

        for coin in &coins[1..] {
            if coin.amount.is_zero() {
                return Err(CoinsError::InvalidAmount);
            };

            // Less than to ensure lexicographical ordering
            // Equality to ensure that there are no duplications
            if coin.denom.to_string() <= previous_denom {
                return Err(CoinsError::DuplicatesOrUnsorted);
            }

            previous_denom = coin.denom.to_string();
        }

        Ok(())
    }

    pub fn into_inner(self) -> Vec<Coin> {
        self.0
    }
}

impl From<SendCoins> for Vec<Coin> {
    fn from(coins: SendCoins) -> Vec<Coin> {
        coins.0
    }
}

impl IntoIterator for SendCoins {
    type Item = Coin;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CoinsParseError {
    #[error("Failed to parse: {0}")]
    Parse(#[from] CoinFromStrError),
    #[error("Parsed invalid coins: {0}")]
    Validate(#[from] CoinsError),
}

impl FromStr for SendCoins {
    type Err = CoinsParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let coin_strings = input.split(',');
        let mut coins = vec![];

        for coin in coin_strings {
            let coin = Coin::from_str(coin)?;
            coins.push(coin);
        }

        Ok(Self::new(coins)?)
    }
}
