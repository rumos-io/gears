use std::str::FromStr;

use proto_types::{error::Error, Denom, Uint256};
use serde::{Deserialize, Serialize};
use tendermint::types::proto::Protobuf;

use super::errors::CoinsError;

mod inner {
    pub use ibc_types::base::coin::Coin;
}

/// Coin defines a token with a denomination and an amount.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Coin {
    pub denom: Denom,
    pub amount: Uint256,
}

impl TryFrom<inner::Coin> for Coin {
    type Error = CoinsError;

    fn try_from(value: inner::Coin) -> Result<Self, Self::Error> {
        let denom = value
            .denom
            .try_into()
            .map_err(|e: Error| CoinsError::Denom(e.to_string()))?;
        let amount =
            Uint256::from_str(&value.amount).map_err(|e| CoinsError::Uint(e.to_string()))?;

        Ok(Coin { denom, amount })
    }
}

impl From<Coin> for inner::Coin {
    fn from(value: Coin) -> inner::Coin {
        Self {
            denom: value.denom.to_string(),
            amount: value.amount.to_string(),
        }
    }
}

impl Protobuf<inner::Coin> for Coin {}

impl FromStr for Coin {
    type Err = CoinsError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // get the index at which amount ends and denom starts
        let i = input.find(|c: char| !c.is_numeric()).unwrap_or(input.len());

        let amount = input[..i]
            .parse::<Uint256>()
            .map_err(|e| CoinsError::Uint(e.to_string()))?;

        let denom = input[i..]
            .parse::<Denom>()
            .map_err(|e| CoinsError::Denom(e.to_string()))?;

        Ok(Coin { denom, amount })
    }
}
