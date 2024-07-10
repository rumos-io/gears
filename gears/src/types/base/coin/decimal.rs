use crate::types::{base::errors::CoinsError, denom::Denom};
use core_types::{errors::CoreError, Protobuf};
use cosmwasm_std::{DecCoin, Decimal256};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::unsigned::Coin;

#[derive(Clone, PartialEq, Eq, Message, Serialize, Deserialize)]
pub struct DecimalCoinRaw {
    #[prost(string, tag = "1")]
    pub denom: String,
    #[prost(string, tag = "2")]
    pub amount: String,
}

impl From<DecimalCoin> for DecimalCoinRaw {
    fn from(DecimalCoin { denom, amount }: DecimalCoin) -> Self {
        Self {
            denom: denom.to_string(),
            amount: amount.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(try_from = "DecimalCoinRaw", into = "DecimalCoinRaw")]
pub struct DecimalCoin {
    pub denom: Denom,
    pub amount: Decimal256, // TODO:LATER DO WE HAVE LOCAL COPY?
}

impl DecimalCoin {
    pub fn new(amount: impl Into<Decimal256>, denom: impl Into<Denom>) -> Self {
        Self {
            denom: denom.into(),
            amount: amount.into(),
        }
    }

    // truncate_decimal returns a Coin with a truncated decimal and a DecimalCoin for the
    // change. Note, the change may be zero.
    pub fn truncate_decimal(self) -> (Coin, DecimalCoin) {
        let truncated = self.amount.to_uint_floor();
        let dec = Decimal256::from_atomics(truncated, 0)
            .expect("cannot fail because it is a truncated part from decimal");
        let change = self.amount - dec;
        (
            Coin {
                amount: truncated,
                denom: self.denom.clone(),
            },
            DecimalCoin::new(change, self.denom.clone()),
        )
    }
}

impl TryFrom<DecCoin> for DecimalCoin {
    type Error = crate::types::errors::Error;

    fn try_from(DecCoin { denom, amount }: DecCoin) -> Result<Self, Self::Error> {
        Ok(Self {
            denom: denom.try_into()?,
            amount,
        })
    }
}

impl TryFrom<DecimalCoinRaw> for DecimalCoin {
    type Error = CoreError;

    fn try_from(DecimalCoinRaw { denom, amount }: DecimalCoinRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            denom: denom
                .try_into()
                .map_err(|e| CoreError::Coin(format!("{e}")))?,
            amount: Decimal256::from_str(&amount).map_err(|e| CoreError::Coin(e.to_string()))?,
        })
    }
}

impl Protobuf<DecimalCoinRaw> for DecimalCoin {}

impl From<DecimalCoin> for DecCoin {
    fn from(DecimalCoin { denom, amount }: DecimalCoin) -> Self {
        Self {
            denom: denom.into_inner(),
            amount,
        }
    }
}

impl FromStr for DecimalCoin {
    type Err = CoinsError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // get the index at which amount ends and denom starts
        let i = input.find(|c: char| !c.is_numeric()).unwrap_or(input.len());

        let amount = input[..i]
            .parse::<Decimal256>()
            .map_err(|e| CoinsError::Uint(e.to_string()))?;

        let denom = input[i..]
            .parse::<Denom>()
            .map_err(|e| CoinsError::Denom(e.to_string()))?;

        Ok(Self { denom, amount })
    }
}

impl From<(Denom, Decimal256)> for DecimalCoin {
    fn from((denom, amount): (Denom, Decimal256)) -> Self {
        Self { denom, amount }
    }
}

impl From<DecimalCoin> for (Denom, Decimal256) {
    fn from(DecimalCoin { denom, amount }: DecimalCoin) -> Self {
        (denom, amount)
    }
}
