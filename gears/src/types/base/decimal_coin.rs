use std::str::FromStr;

use cosmwasm_std::{DecCoin, Decimal256};
use serde::{Deserialize, Serialize};

use crate::types::denom::Denom;

use super::errors::CoinsError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
