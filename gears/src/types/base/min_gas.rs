use std::str::FromStr;

use cosmwasm_std::Decimal256;
use serde::{Deserialize, Serialize};

use crate::types::denom::Denom;

use super::{
    coin::DecimalCoin,
    coins::DecimalCoins,
    errors::{CoinsError, CoinsParseError},
};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MinGasPrices(DecimalCoins);

impl Default for MinGasPrices {
    fn default() -> Self {
        Self(
            DecimalCoins::new(vec![DecimalCoin {
                denom: Denom::from_str("uatom").expect("Default is valid"),
                amount: Decimal256::zero(),
            }])
            .expect("Default is valid"),
        )
    }
}

impl MinGasPrices {
    pub fn new(coins: Vec<DecimalCoin>) -> Result<Self, CoinsError> {
        Ok(Self(DecimalCoins::new(coins)?))
    }

    pub fn into_inner(self) -> Vec<DecimalCoin> {
        self.0.into_inner()
    }

    pub fn inner(&self) -> &Vec<DecimalCoin> {
        self.0.inner()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn len(&self) -> usize {
        self.0.len()
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
