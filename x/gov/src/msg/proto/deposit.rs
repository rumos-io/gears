use bytes::Bytes;
use gears::{
    core::{any::google::Any, base::coin::Coin, errors::CoreError},
    tendermint::types::proto::Protobuf,
    types::{
        address::AccAddress,
        base::{errors::CoinsError, send::SendCoins},
        tx::TxMessage,
    },
};
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::msg::deposit::Deposit;

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct MsgDeposit {
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    #[prost(string, tag = "2")]
    pub depositor: String,
    #[prost(message, repeated, tag = "3")]
    pub amount: Vec<Coin>,
}

impl TxMessage for MsgDeposit {
    fn get_signers(&self) -> Vec<&AccAddress> {
        Vec::new() // Todo:NOT what and how?
    }

    fn validate_basic(&self) -> Result<(), String> {
        Ok(())
    }

    fn type_url(&self) -> &'static str {
        "/cosmos.gov.v1beta1/MsgDeposit"
    }
}

impl Protobuf<MsgDeposit> for Deposit {}

impl TryFrom<MsgDeposit> for Deposit {
    type Error = CoreError;

    fn try_from(
        MsgDeposit {
            proposal_id,
            depositor,
            amount,
        }: MsgDeposit,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            depositor: AccAddress::from_bech32(&depositor)
                .map_err(|e| CoreError::Coins(e.to_string()))?,
            amount: {
                let mut coins = Vec::with_capacity(amount.len());
                for coin in amount {
                    coins.push(
                        coin.try_into()
                            .map_err(|e: CoinsError| CoreError::Coin(e.to_string()))?,
                    )
                }
                SendCoins::new(coins).map_err(|e| CoreError::DecodeAddress(e.to_string()))?
            },
        })
    }
}

impl From<Deposit> for MsgDeposit {
    fn from(
        Deposit {
            proposal_id,
            depositor,
            amount,
        }: Deposit,
    ) -> Self {
        Self {
            proposal_id,
            depositor: depositor.into(),
            amount: amount
                .into_inner()
                .into_iter()
                .map(|this| this.into())
                .collect(),
        }
    }
}

impl TryFrom<Any> for MsgDeposit {
    type Error = CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        Ok(MsgDeposit::decode::<Bytes>(value.value.into())?)
    }
}

impl From<MsgDeposit> for Any {
    fn from(msg: MsgDeposit) -> Self {
        Any {
            type_url: "/TODO".to_string(),
            value: msg.encode_to_vec(),
        }
    }
}
