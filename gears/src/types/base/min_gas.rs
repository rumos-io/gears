use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::{
    coin::DecimalCoin,
    errors::{CoinsParseError, SendCoinsError},
};

// Represents a list of coins with the following properties:
// - Contains at least one coin
// - No duplicate denominations
// - Sorted lexicographically
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct MinGasPrices(Vec<DecimalCoin>);

impl MinGasPrices {
    pub fn new(coins: Vec<DecimalCoin>) -> Result<Self, SendCoinsError> {
        Self::validate_coins(&coins)?;

        Ok(Self(coins))
    }

    // Checks that the SendCoins are sorted, have positive amount, with a valid and unique
    // denomination (i.e no duplicates). Otherwise, it returns an error.
    // A valid list of coins satisfies:
    // - Contains at least one coin
    // - No duplicate denominations
    // - Sorted lexicographically
    // TODO: implement ordering on coins or denominations so that conversion to string can be avoided
    fn validate_coins(coins: &Vec<DecimalCoin>) -> Result<(), SendCoinsError> {
        if coins.is_empty() {
            return Err(SendCoinsError::EmptyList);
        }

        let mut previous_denom = coins[0].denom.to_string();

        for coin in &coins[1..] {
            // Less than to ensure lexicographical ordering
            // Equality to ensure that there are no duplications
            if coin.denom.to_string() <= previous_denom {
                return Err(SendCoinsError::DuplicatesOrUnsorted);
            }

            previous_denom = coin.denom.to_string();
        }

        Ok(())
    }

    pub fn into_inner(self) -> Vec<DecimalCoin> {
        self.0
    }

    pub fn inner(&self) -> &Vec<DecimalCoin> {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().any(|this| this.amount.is_zero())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<MinGasPrices> for Vec<DecimalCoin> {
    fn from(coins: MinGasPrices) -> Vec<DecimalCoin> {
        coins.0
    }
}

impl IntoIterator for MinGasPrices {
    type Item = DecimalCoin;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromStr for MinGasPrices {
    type Err = CoinsParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let coin_strings = input.split(',');
        let mut coins = vec![];

        for coin in coin_strings {
            let coin = DecimalCoin::from_str(coin)?;
            coins.push(coin);
        }

        Ok(Self::new(coins)?)
    }
}
