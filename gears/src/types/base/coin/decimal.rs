use crate::types::{base::errors::CoinError, denom::Denom, errors::DenomError};
use core_types::{errors::CoreError, Protobuf};
use cosmwasm_std::{DecCoin, Decimal256};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::{unsigned::UnsignedCoin, Coin};

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

impl Coin for DecimalCoin {
    type Amount = Decimal256;

    fn denom(&self) -> &Denom {
        &self.denom
    }

    fn amount(&self) -> &Decimal256 {
        &self.amount
    }
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
    pub fn truncate_decimal(&self) -> (UnsignedCoin, DecimalCoin) {
        let truncated = self.amount.to_uint_floor();
        let dec = Decimal256::from_atomics(truncated, 0)
            .expect("cannot fail because it is a truncated part from decimal");
        let change = self.amount - dec;
        (
            UnsignedCoin {
                amount: truncated,
                denom: self.denom.clone(),
            },
            DecimalCoin::new(change, self.denom.clone()),
        )
    }
}

impl TryFrom<DecCoin> for DecimalCoin {
    type Error = DenomError;

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
    type Err = CoinError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // get the index at which amount ends and denom starts
        let i = input.find(|c: char| !c.is_numeric()).unwrap_or(input.len());

        let amount = input[..i]
            .parse::<Decimal256>()
            .map_err(|e| CoinError::Decimal(e.to_string()))?;

        let denom = input[i..]
            .parse::<Denom>()
            .map_err(|e| CoinError::Denom(e.to_string()))?;

        Ok(Self { denom, amount })
    }
}

impl std::fmt::Display for DecimalCoin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.amount, self.denom)
    }
}
