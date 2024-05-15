use serde::{Deserialize, Serialize};
use tendermint::types::proto::Protobuf;

use crate::types::{
    address::{AccAddress, AddressError},
    base::{
        coin::Coin,
        errors::{CoinsError, SendCoinsError},
        send::SendCoins,
    },
};

mod inner {
    pub use core_types::auth::tip::Tip;
    pub use core_types::base::coin::Coin;
}

// Tip is the tip used for meta-transactions.
//
// Since: cosmos-sdk 0.46
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tip {
    /// amount is the amount of the tip
    pub amount: Option<SendCoins>,
    /// tipper is the address of the account paying for the tip
    pub tipper: AccAddress,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TipError {
    #[error("{0}")]
    Address(#[from] AddressError),
    #[error("{0}")]
    SendError(#[from] SendCoinsError),
    #[error("{0}")]
    Coins(#[from] CoinsError),
}

impl TryFrom<inner::Tip> for Tip {
    type Error = TipError;

    fn try_from(raw: inner::Tip) -> Result<Self, Self::Error> {
        let tipper = AccAddress::from_bech32(&raw.tipper)?;

        let coins: Result<Vec<Coin>, CoinsError> =
            raw.amount.into_iter().map(Coin::try_from).collect();

        Ok(Tip {
            amount: Some(SendCoins::new(coins?)?),
            tipper,
        })
    }
}

impl From<Tip> for inner::Tip {
    fn from(tip: Tip) -> inner::Tip {
        let tipper = tip.tipper.to_string();

        match tip.amount {
            Some(amount) => {
                let coins: Vec<Coin> = amount.into();
                let coins = coins.into_iter().map(inner::Coin::from).collect();

                Self {
                    amount: coins,
                    tipper,
                }
            }
            None => Self {
                amount: vec![],
                tipper,
            },
        }
    }
}

impl Protobuf<inner::Tip> for Tip {}
