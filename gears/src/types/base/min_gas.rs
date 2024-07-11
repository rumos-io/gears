use std::str::FromStr;

use cosmwasm_std::Decimal256;
use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::types::denom::Denom;

use super::{
    coin::DecimalCoin,
    errors::{CoinsError, CoinsParseError},
};

#[derive(Clone, PartialEq, Debug, SerializeDisplay, DeserializeFromStr)]
pub struct MinGasPrices(Vec<DecimalCoin>);

impl std::fmt::Display for MinGasPrices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let last = self.0.last();

        for coin in &self.0 {
            if let Some(last) = last {
                if last == coin {
                    write!(f, "{}{}", last.amount, last.denom)?;
                } else {
                    write!(f, "{}{},", coin.amount, coin.denom)?;
                }
            }
        }

        std::fmt::Result::Ok(())
    }
}

impl Default for MinGasPrices {
    fn default() -> Self {
        Self(vec![DecimalCoin {
            denom: Denom::from_str("uatom").expect("Default is valid"),
            amount: Decimal256::zero(),
        }])
    }
}

impl MinGasPrices {
    pub fn new(coins: impl IntoIterator<Item = DecimalCoin>) -> Result<Self, CoinsError> {
        let coins = coins.into_iter().collect::<Vec<_>>();

        if coins.is_empty() {
            Err(CoinsError::EmptyList)?
        }

        {
            let mut previous_denom = &coins[0].denom;

            for coin in &coins[1..] {
                // Less than to ensure lexicographical ordering
                // Equality to ensure that there are no duplications
                match coin.denom.cmp(&previous_denom) {
                    std::cmp::Ordering::Less => Err(CoinsError::Unsorted),
                    std::cmp::Ordering::Equal => Err(CoinsError::Duplicates),
                    std::cmp::Ordering::Greater => Ok(()),
                }?;

                previous_denom = &coin.denom;
            }
        }

        Ok(Self(coins))
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
