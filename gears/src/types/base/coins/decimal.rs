use crate::types::{
    base::{
        coin::{Coin, DecimalCoin},
        errors::{CoinsParseError, SendCoinsError},
    },
    denom::Denom,
};
use cosmwasm_std::Decimal256;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

use super::unsigned::Coins;

// Represents a list of coins with the following properties:
// - Contains at least one coin
// - All coin amounts are positive
// - No duplicate denominations
// - Sorted lexicographically
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DecimalCoins(Vec<DecimalCoin>);

impl DecimalCoins {
    pub fn new(coins: Vec<DecimalCoin>) -> Result<DecimalCoins, SendCoinsError> {
        Self::validate_coins(&coins)?;
        Ok(DecimalCoins(coins))
    }

    // Checks that the DecimalCoins are sorted, have positive amount, with a valid and unique
    // denomination (i.e no duplicates). Otherwise, it returns an error.
    // A valid list of coins satisfies:
    // - Contains at least one coin
    // - All amounts are positive
    // - No duplicate denominations
    // - Sorted lexicographically
    // TODO: implement ordering on coins or denominations so that conversion to string can be avoided
    fn validate_coins(coins: &[DecimalCoin]) -> Result<(), SendCoinsError> {
        if coins.is_empty() {
            return Err(SendCoinsError::EmptyList);
        }

        if coins[0].amount.is_zero() {
            return Err(SendCoinsError::InvalidAmount);
        };

        let mut previous_denom = coins[0].denom.to_string();

        for coin in &coins[1..] {
            if coin.amount.is_zero() {
                return Err(SendCoinsError::InvalidAmount);
            };

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

    pub fn amount_of(&self, denom: &Denom) -> Decimal256 {
        let coins = self
            .0
            .iter()
            .map(|this| (&this.denom, &this.amount))
            .collect::<HashMap<_, _>>();

        if let Some(coin) = coins.get(denom) {
            **coin
        } else {
            Decimal256::zero()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn checked_add(&self, other: DecimalCoins) -> Result<Self, SendCoinsError> {
        let result = self
            .inner()
            .iter()
            .cloned()
            .chain(other.0)
            .collect::<Vec<_>>();

        Self::new(result)
    }

    pub fn is_all_gte<'a>(&self, other: impl IntoIterator<Item = &'a DecimalCoin>) -> bool {
        let other = other.into_iter().collect::<Vec<_>>();

        if other.is_empty() {
            return true;
        }

        for coin in other {
            if coin.amount >= self.amount_of(&coin.denom) {
                return false;
            }
        }

        true
    }

    pub fn truncate_decimal(&self) -> (Coins, DecimalCoins) {
        let (truncated, change): (Vec<Coin>, Vec<DecimalCoin>) =
            self.0.iter().map(DecimalCoin::truncate_decimal).unzip();

        (
            Coins::new(
                truncated
                    .into_iter()
                    .filter(|c| !c.amount.is_zero())
                    .collect(),
            )
            .expect("inner structure of coins should be unchanged"),
            DecimalCoins::new(change.into_iter().filter(|c| !c.amount.is_zero()).collect())
                .expect("inner structure of coins should be unchanged"),
        )
    }
}

impl From<DecimalCoins> for Vec<DecimalCoin> {
    fn from(coins: DecimalCoins) -> Vec<DecimalCoin> {
        coins.0
    }
}

impl IntoIterator for DecimalCoins {
    type Item = DecimalCoin;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromStr for DecimalCoins {
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
