use ibc_proto::{cosmos::base::v1beta1::Coin as RawCoin, protobuf::Protobuf};

pub use ibc_proto::cosmos::tx::v1beta1::Tip as RawTip;

use proto_types::AccAddress;
use serde::{Deserialize, Serialize};

use crate::{
    cosmos::base::v1beta1::{Coin, SendCoins},
    error::Error,
};

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

impl TryFrom<RawTip> for Tip {
    type Error = Error;

    fn try_from(raw: RawTip) -> Result<Self, Self::Error> {
        let tipper = AccAddress::from_bech32(&raw.tipper)
            .map_err(|e| Error::DecodeAddress(e.to_string()))?;

        let coins: Result<Vec<Coin>, Error> = raw.amount.into_iter().map(Coin::try_from).collect();

        Ok(Tip {
            amount: Some(SendCoins::new(coins?)?),
            tipper,
        })
    }
}

impl From<Tip> for RawTip {
    fn from(tip: Tip) -> RawTip {
        let tipper = tip.tipper.to_string();

        match tip.amount {
            Some(amount) => {
                let coins: Vec<Coin> = amount.into();
                let coins = coins.into_iter().map(RawCoin::from).collect();

                RawTip {
                    amount: coins,
                    tipper,
                }
            }
            None => RawTip {
                amount: vec![],
                tipper,
            },
        }
    }
}

impl Protobuf<RawTip> for Tip {}
