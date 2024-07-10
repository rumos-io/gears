pub use decimal::*;
pub use unsigned::*;

mod decimal;
mod unsigned;

use std::{collections::BTreeMap, marker::PhantomData, str::FromStr};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::types::denom::Denom;

use super::{
    errors::{CoinError, CoinsError, CoinsParseError},
    ZeroNumeric,
};

#[derive(Serialize, Deserialize)]
pub struct CoinsRaw<U>(Vec<U>);

impl<T: Clone + ZeroNumeric, U: Into<(Denom, T)> + From<(Denom, T)> + Clone> From<Coins<T, U>>
    for CoinsRaw<U>
{
    fn from(Coins { storage, _marker }: Coins<T, U>) -> Self {
        Self(storage.into_iter().map(|this| U::from(this)).collect())
    }
}

impl<T: Clone + ZeroNumeric, U: Into<(Denom, T)> + From<(Denom, T)> + Clone> TryFrom<CoinsRaw<U>>
    for Coins<T, U>
{
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
pub struct Coins<T: Clone + ZeroNumeric, U: Into<(Denom, T)> + From<(Denom, T)> + Clone> {
    storage: BTreeMap<Denom, T>,
    _marker: PhantomData<U>,
}

impl<T: Clone + ZeroNumeric, U: Into<(Denom, T)> + From<(Denom, T)> + Clone> Coins<T, U> {
    // Checks that the SendCoins are sorted, have positive amount, with a valid and unique
    // denomination (i.e no duplicates). Otherwise, it returns an error.
    // A valid list of coins satisfies:
    // - Contains at least one coin
    // - All amounts are positive
    // - No duplicate denominations
    // - Sorted lexicographically
    pub fn new(coins: impl IntoIterator<Item = U>) -> Result<Self, CoinsError> {
        Self::try_new(coins.into_iter().map(|this| this.into()))
    }

    fn try_new(coins: impl IntoIterator<Item = (Denom, T)>) -> Result<Self, CoinsError> {
        let coins = coins.into_iter().collect::<Vec<_>>();

        if coins.is_empty() {
            Err(CoinsError::EmptyList)?
        }

        if coins.iter().any(|this| this.1.is_zero()) {
            Err(CoinsError::InvalidAmount)?
        }

        let mut storage = BTreeMap::<Denom, T>::new();

        for (denom, amount) in coins {
            if storage.contains_key(&denom) {
                Err(CoinsError::Duplicates)?
            } else {
                storage.insert(denom, amount);
            }
        }

        Ok(Self {
            storage,
            _marker: PhantomData,
        })
    }

    pub fn amount_of(&self, denom: &Denom) -> T {
        match self.storage.get(denom) {
            Some(amount) => amount.clone(),
            None => T::zero(),
        }
    }

    pub fn checked_add(&self, other: Self) -> Result<Self, CoinsError> {
        let result = self
            .storage
            .iter()
            .map(|(denom, amount)| (denom.clone(), amount.clone()))
            .chain(other.storage);

        Self::try_new(result)
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    pub fn into_inner(self) -> Vec<U> {
        self.storage.into_iter().map(|this| U::from(this)).collect()
    }

    pub fn first(&self) -> U {
        let coin = self
            .storage
            .first_key_value()
            .map(|(denom, amount)| (denom.clone(), amount.clone()))
            .expect("Should contains at least single element");

        U::from(coin)
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }
}

impl<
        T: ZeroNumeric + Clone,
        U: FromStr<Err = CoinError> + Into<(Denom, T)> + From<(Denom, T)> + Clone,
    > FromStr for Coins<T, U>
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

impl<T: ZeroNumeric + Clone, U: Into<(Denom, T)> + From<(Denom, T)> + Clone> From<Coins<T, U>>
    for Vec<U>
{
    fn from(coins: Coins<T, U>) -> Vec<U> {
        coins
            .storage
            .into_iter()
            .map(|coin| U::from(coin))
            .collect()
    }
}

impl<T: ZeroNumeric + Clone, U: Into<(Denom, T)> + From<(Denom, T)> + Clone> IntoIterator
    for Coins<T, U>
{
    type Item = U;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.storage
            .into_iter()
            .map(|coin| U::from(coin))
            .collect::<Vec<_>>()
            .into_iter()
    }
}
