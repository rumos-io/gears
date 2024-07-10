use gears::{
    core::{errors::CoreError, Protobuf},
    types::base::{
        coin::{DecimalCoin, DecimalCoinRaw},
        coins::DecimalCoins,
        errors::SendCoinsError,
    },
};
use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Message)]
pub struct FeePoolRaw {
    #[prost(message, repeated, tag = "1")]
    pub community_pool: Vec<DecimalCoinRaw>,
}

impl From<FeePool> for FeePoolRaw {
    fn from(FeePool { community_pool }: FeePool) -> Self {
        Self {
            community_pool: community_pool
                .into_inner()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

/// FeePool is the global fee pool for distribution.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(try_from = "Vec<DecimalCoin>")]
pub struct FeePool {
    pub community_pool: DecimalCoins,
}

impl TryFrom<Vec<DecimalCoin>> for FeePool {
    type Error = SendCoinsError;

    fn try_from(value: Vec<DecimalCoin>) -> Result<Self, Self::Error> {
        Ok(Self {
            community_pool: DecimalCoins::new(value)?,
        })
    }
}

impl TryFrom<FeePoolRaw> for FeePool {
    type Error = CoreError;

    fn try_from(FeePoolRaw { community_pool }: FeePoolRaw) -> Result<Self, Self::Error> {
        let mut coins = vec![];
        for coin in community_pool {
            coins.push(coin.try_into()?);
        }
        let community_pool =
            DecimalCoins::new(coins).map_err(|e| CoreError::Coin(e.to_string()))?;
        Ok(Self { community_pool })
    }
}

impl Protobuf<FeePoolRaw> for FeePool {}
