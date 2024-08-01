use bytes::Bytes;
use core_types::any::google::Any;
use serde::{Deserialize, Serialize};
use tendermint::types::proto::Protobuf;

use crate::types::{
    address::AccAddress,
    base::{coin::UnsignedCoin, coins::UnsignedCoins, errors::CoinError},
    tx::TxMessage,
};

mod inner {
    pub use core_types::base::coin::Coin;
    pub use core_types::msg::MsgSend;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
#[error("MsgSend parse error: {0}")]
pub struct MsgSendParseError(pub String);

/// MsgSend represents a message to send coins from one account to another.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MsgSend {
    pub from_address: AccAddress,
    pub to_address: AccAddress,
    pub amount: UnsignedCoins,
}

impl TryFrom<inner::MsgSend> for MsgSend {
    type Error = MsgSendParseError;

    fn try_from(raw: inner::MsgSend) -> Result<Self, Self::Error> {
        let from_address = AccAddress::from_bech32(&raw.from_address)
            .map_err(|e| MsgSendParseError(e.to_string()))?;

        let to_address = AccAddress::from_bech32(&raw.to_address)
            .map_err(|e| MsgSendParseError(e.to_string()))?;

        let coins = raw
            .amount
            .into_iter()
            .map(UnsignedCoin::try_from)
            .collect::<Result<Vec<_>, CoinError>>()
            .map_err(|e| MsgSendParseError(e.to_string()))?;

        Ok(MsgSend {
            from_address,
            to_address,
            amount: UnsignedCoins::new(coins).map_err(|e| MsgSendParseError(e.to_string()))?,
        })
    }
}

impl From<MsgSend> for inner::MsgSend {
    fn from(msg: MsgSend) -> inner::MsgSend {
        let coins: Vec<UnsignedCoin> = msg.amount.into();
        let coins = coins.into_iter().map(inner::Coin::from).collect();

        Self {
            from_address: msg.from_address.into(),
            to_address: msg.to_address.into(),
            amount: coins,
        }
    }
}

impl Protobuf<inner::MsgSend> for MsgSend {}

//TODO: should to Any be implemented at the individual message type?
impl From<MsgSend> for Any {
    fn from(msg: MsgSend) -> Self {
        Any {
            type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
            value: msg.encode_vec().expect("msg"), // TODO
        }
    }
}

impl TxMessage for MsgSend {
    fn get_signers(&self) -> Vec<&AccAddress> {
        todo!()
    }

    fn type_url(&self) -> &'static str {
        "/cosmos.bank.v1beta1.MsgSend"
    }
}

impl TryFrom<Any> for MsgSend {
    type Error = core_types::errors::CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        MsgSend::decode::<Bytes>(value.value.clone().into())
            .map_err(|e| core_types::errors::CoreError::DecodeAny(e.to_string()))
    }
}
