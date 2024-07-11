pub use decimal::*;
pub use unsigned::*;

mod decimal;
mod unsigned;

use std::{marker::PhantomData, str::FromStr};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::types::denom::Denom;

use super::{
    coin::Coin,
    errors::{CoinError, CoinsError, CoinsParseError},
    ZeroNumeric,
};

#[derive(Serialize, Deserialize)]
pub struct CoinsRaw<U>(Vec<U>);

impl<T: ZeroNumeric, U: Coin<Amount = T>> From<Coins<T, U>> for CoinsRaw<U> {
    fn from(Coins { storage, _marker }: Coins<T, U>) -> Self {
        Self(storage.into_iter().collect())
    }
}

impl<T: Clone + ZeroNumeric, U: Coin<Amount = T>> TryFrom<CoinsRaw<U>> for Coins<T, U> {
    type Error = CoinsError;

    fn try_from(CoinsRaw(value): CoinsRaw<U>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

// Represents a list of coins with the following properties:
// - Contains at least one coin
// - All coin amounts are positive
// - No duplicate denominations
// - Sorted lexicographically
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(try_from = "CoinsRaw<U>", into = "CoinsRaw<U>")]
#[serde(bound = "U: Serialize + DeserializeOwned")]
pub struct Coins<T: ZeroNumeric, U: Coin<Amount = T>> {
    storage: Vec<U>,
    _marker: PhantomData<T>,
}

impl<T: ZeroNumeric, U: Coin<Amount = T>> Coins<T, U> {
    // Checks that the Coins are sorted, have positive amount, with a valid and unique
    // denomination (i.e no duplicates). Otherwise, it returns an error.
    // A valid list of coins satisfies:
    // - Contains at least one coin
    // - All amounts are positive
    // - No duplicate denominations
    // - Sorted lexicographically
    pub fn new(coins: impl IntoIterator<Item = U>) -> Result<Self, CoinsError> {
        let coins = coins.into_iter().collect::<Vec<_>>();

        if coins.is_empty() {
            Err(CoinsError::EmptyList)?
        }

        if coins.iter().any(|this| this.amount().is_zero()) {
            Err(CoinsError::InvalidAmount)?
        }

        {
            let mut previous_denom = coins[0].denom();

            for coin in &coins[1..] {
                // Less than to ensure lexicographical ordering
                // Equality to ensure that there are no duplications
                match coin.denom().cmp(previous_denom) {
                    std::cmp::Ordering::Less => Err(CoinsError::Unsorted),
                    std::cmp::Ordering::Equal => Err(CoinsError::Duplicates),
                    std::cmp::Ordering::Greater => Ok(()),
                }?;

                previous_denom = coin.denom();
            }
        }

        Ok(Self {
            storage: coins,
            _marker: PhantomData,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    pub fn inner(&self) -> &Vec<U> {
        &self.storage
    }

    pub fn into_inner(self) -> Vec<U> {
        self.storage
    }

    pub fn amount_of(&self, denom: &Denom) -> T {
        match self.storage.iter().find(|this| this.denom() == denom) {
            Some(coin) => coin.amount().clone(),
            None => T::zero(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.storage.iter().all(|this| this.amount().is_zero())
    }

    pub fn first(&self) -> U {
        self.storage
            .first()
            .cloned()
            .expect("Should contains at least single element")
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }
}

impl<T: ZeroNumeric + Clone, U: FromStr<Err = CoinError> + Coin<Amount = T>> FromStr
    for Coins<T, U>
{
    type Err = CoinsParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let coin_strings = input.split(',');
        let mut coins = vec![];

        for coin in coin_strings {
            let coin = U::from_str(coin)?;
            coins.push(coin);
        }

        Ok(Self::new(coins)?)
    }
}

impl<T: ZeroNumeric + Clone, U: Coin<Amount = T>> From<Coins<T, U>> for Vec<U> {
    fn from(coins: Coins<T, U>) -> Vec<U> {
        coins.storage
    }
}

impl<T: ZeroNumeric + Clone, U: Coin<Amount = T>> IntoIterator for Coins<T, U> {
    type Item = U;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.storage.into_iter()
    }
}
