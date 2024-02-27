use ibc_proto::{
    cosmos::bank::v1beta1::MsgSend as RawMsgSend, cosmos::base::v1beta1::Coin as RawCoin, Protobuf,
};
use prost::bytes::Bytes;
use proto_types::AccAddress;
use serde::{Deserialize, Serialize};

use crate::{
    any::Any, cosmos::{
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::message::Message,
    }, error::Error
};

/// MsgSend represents a message to send coins from one account to another.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MsgSend {
    pub from_address: AccAddress,
    pub to_address: AccAddress,
    pub amount: SendCoins,
}

impl TryFrom<RawMsgSend> for MsgSend {
    type Error = Error;

    fn try_from(raw: RawMsgSend) -> Result<Self, Self::Error> {
        let from_address = AccAddress::from_bech32(&raw.from_address)
            .map_err(|e| Error::DecodeAddress(e.to_string()))?;

        let to_address = AccAddress::from_bech32(&raw.to_address)
            .map_err(|e| Error::DecodeAddress(e.to_string()))?;

        let coins: Result<Vec<Coin>, Error> = raw.amount.into_iter().map(Coin::try_from).collect();

        Ok(MsgSend {
            from_address,
            to_address,
            amount: SendCoins::new(coins?)?,
        })
    }
}

impl From<MsgSend> for RawMsgSend {
    fn from(msg: MsgSend) -> RawMsgSend {
        let coins: Vec<Coin> = msg.amount.into();
        let coins = coins.into_iter().map(RawCoin::from).collect();

        RawMsgSend {
            from_address: msg.from_address.into(),
            to_address: msg.to_address.into(),
            amount: coins,
        }
    }
}

impl Protobuf<RawMsgSend> for MsgSend {}

//TODO: should to Any be implemented at the individual message type?
impl From<MsgSend> for Any {
    fn from(msg: MsgSend) -> Self {
        Any {
            type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
            value: msg.encode_vec(),
        }
    }
}

impl Message for MsgSend {
    fn get_signers(&self) -> Vec<&AccAddress> {
        todo!()
    }

    fn validate_basic(&self) -> Result<(), String> {
        todo!()
    }

    fn type_url(&self) -> &'static str {
        "/cosmos.bank.v1beta1.MsgSend"
    }
}

impl TryFrom<Any> for MsgSend {
    type Error = Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        MsgSend::decode::<Bytes>(value.value.clone().into())
            .map_err(|e| Error::DecodeAny(e.to_string()))
    }
}
