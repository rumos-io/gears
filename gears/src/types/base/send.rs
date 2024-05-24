use std::{collections::HashMap, str::FromStr};

use cosmwasm_std::Uint256;
use serde::{Deserialize, Serialize};

use crate::types::denom::Denom;

use super::{
    coin::Coin,
    errors::{CoinsParseError, SendCoinsError},
};

// Represents a list of coins with the following properties:
// - Contains at least one coin
// - All coin amounts are positive
// - No duplicate denominations
// - Sorted lexicographically
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct SendCoins(Vec<Coin>);

impl SendCoins {
    pub fn new(coins: Vec<Coin>) -> Result<SendCoins, SendCoinsError> {
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
    fn validate_coins(coins: &Vec<Coin>) -> Result<(), SendCoinsError> {
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

    pub fn into_inner(self) -> Vec<Coin> {
        self.0
    }

    pub fn inner(&self) -> &Vec<Coin> {
        &self.0
    }

    pub fn amount_of(&self, denom: &Denom) -> Uint256 {
        let coins = self
            .0
            .iter()
            .map(|this| (&this.denom, &this.amount))
            .collect::<HashMap<_, _>>();

        let coin = coins.get(denom);

        if let Some(coin) = coin {
            **coin
        } else {
            Uint256::zero()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
