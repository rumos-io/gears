use super::{
    coin::Coin,
    decimal_coin::DecimalCoin,
    errors::{CoinsParseError, SendCoinsError},
    send::SendCoins,
};
use crate::types::denom::Denom;
use cosmwasm_std::{Decimal256, OverflowError};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap, str::FromStr};

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

    pub fn checked_add(&self, other: &DecimalCoins) -> Result<Self, SendCoinsError> {
        let coins = self.checked_calculate_iterate(other.inner(), Decimal256::checked_add)?;
        Self::new(coins)
    }

    fn checked_calculate_iterate(
        &self,
        other_coins: &[DecimalCoin],
        operation: impl Fn(Decimal256, Decimal256) -> Result<Decimal256, OverflowError>,
    ) -> Result<Vec<DecimalCoin>, SendCoinsError> {
        let mut i = 0;
        let mut j = 0;
        let self_coins = self.inner();

        let mut result = vec![];
        // we do not handle cases where length of a coins is 0, because DecimalCoins is non-empty
        // list
        while i < self_coins.len() || j < other_coins.len() {
            match self_coins[i].denom.cmp(&other_coins[j].denom) {
                Ordering::Less => {
                    result.push(self_coins[i].clone());
                    i += 1;
                }
                Ordering::Equal => {
                    result.push(DecimalCoin {
                        denom: self_coins[i].denom.clone(),
                        amount: operation(self_coins[i].amount, other_coins[j].amount)
                            .map_err(|_| SendCoinsError::InvalidAmount)?,
                    });
                    i += 1;
                    j += 1;
                }
                Ordering::Greater => {
                    result.push(other_coins[j].clone());
                    j += 1;
                }
            }
        }

        Ok(result)
    }

    pub fn checked_sub(&self, other: &DecimalCoins) -> Result<Self, SendCoinsError> {
        let coins = self.checked_calculate_iterate(other.inner(), Decimal256::checked_sub)?;
        Self::new(coins)
    }

    pub fn checked_mul_dec_truncate(&self, multiplier: Decimal256) -> Result<Self, SendCoinsError> {
        let mut coins = vec![];
        for coin in self.inner().iter() {
            coins.push(DecimalCoin::new(
                coin.amount
                    .checked_mul(multiplier)
                    // TODO: extend error
                    .map_err(|_| SendCoinsError::InvalidAmount)?
                    .floor(),
                coin.denom.clone(),
            ));
        }

        Self::new(coins)
    }

    pub fn checked_mul_dec(&self, multiplier: Decimal256) -> Result<Self, SendCoinsError> {
        let mut coins = vec![];
        for coin in self.inner().iter() {
            let normal = DecimalCoin::new(
                coin.amount
                    .checked_mul(multiplier)
                    .map_err(|_| SendCoinsError::InvalidAmount)?
                    .floor(),
                coin.denom.clone(),
            );
            let mut floored = DecimalCoin::new(
                coin.amount
                    .checked_mul(multiplier)
                    .map_err(|_| SendCoinsError::InvalidAmount)?
                    .floor(),
                coin.denom.clone(),
            );

            if normal.amount - floored.amount
                >= Decimal256::from_atomics(5u64, 0).expect("hardcoded values cannot fail")
            {
                floored.amount += Decimal256::one();
            }
            coins.push(floored);
        }

        Self::new(coins)
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

    pub fn truncate_decimal(&self) -> (SendCoins, DecimalCoins) {
        let (truncated, change): (Vec<Coin>, Vec<DecimalCoin>) =
            self.0.iter().map(DecimalCoin::truncate_decimal).unzip();

        (
            SendCoins::new(
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

// TODO: maybe SendCoins instead of Vec<DecimalCoin
impl TryFrom<Vec<Coin>> for DecimalCoins {
    type Error = SendCoinsError;

    fn try_from(value: Vec<Coin>) -> Result<Self, Self::Error> {
        // TODO: maybe add TryFrom<Coin> for DecimalCoin
        let mut dec_coins = vec![];
        for coin in value {
            let amount = Decimal256::from_atomics(coin.amount, 0)
                .map_err(|_| SendCoinsError::InvalidAmount)?;
            dec_coins.push(DecimalCoin::new(amount, coin.denom));
        }
        Self::new(dec_coins)
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
