use bytes::Bytes;
use gears::core::any::google::Any;
use gears::core::Protobuf;
use gears::types::address::AccAddress;
use gears::types::tx::TxMessage;
use serde::Serialize;

use crate::{MsgFundCommunityPool, MsgSetWithdrawAddr, MsgWithdrawDelegatorReward};

#[derive(Debug, Clone, Serialize)]
pub enum Message {
    #[serde(rename = "/cosmos.distribution.v1beta1.WithdrawRewards")]
    WithdrawRewards(MsgWithdrawDelegatorReward),
    #[serde(rename = "/cosmos.distribution.v1beta1.SetWithdrawAddr")]
    SetWithdrawAddr(MsgSetWithdrawAddr),
    #[serde(rename = "/cosmos.distribution.v1beta1.FundCommunityPool")]
    FundCommunityPool(MsgFundCommunityPool),
}

impl TxMessage for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match self {
            Message::WithdrawRewards(msg) => vec![&msg.delegator_address],
            Message::SetWithdrawAddr(msg) => vec![&msg.delegator_address],
            Message::FundCommunityPool(msg) => vec![&msg.depositor],
        }
    }

    fn validate_basic(&self) -> Result<(), String> {
        Ok(())
    }

    fn type_url(&self) -> &'static str {
        match self {
            Message::WithdrawRewards(_) => "/cosmos.distribution.v1beta1.WithdrawRewards",
            Message::SetWithdrawAddr(_) => "/cosmos.distribution.v1beta1.SetWithdrawAddr",
            Message::FundCommunityPool(_) => "/cosmos.distribution.v1beta1.FundCommunityPool",
        }
    }
}

impl From<Message> for Any {
    fn from(msg: Message) -> Self {
        match msg {
            Message::WithdrawRewards(msg) => Any {
                type_url: "/cosmos.distribution.v1beta1.WithdrawRewards".to_string(),
                value: msg.encode_vec(),
            },
            Message::SetWithdrawAddr(msg) => Any {
                type_url: "/cosmos.distribution.v1beta1.SetWithdrawAddr".to_string(),
                value: msg.encode_vec(),
            },
            Message::FundCommunityPool(msg) => Any {
                type_url: "/cosmos.distribution.v1beta1.FundCommunityPool".to_string(),
                value: msg.encode_vec(),
            },
        }
    }
}

impl TryFrom<Any> for Message {
    type Error = gears::core::errors::CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "/cosmos.distribution.v1beta1.WithdrawRewards" => {
                let msg = MsgWithdrawDelegatorReward::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| {
                    gears::core::errors::CoreError::DecodeProtobuf(e.to_string())
                })?;
                Ok(Message::WithdrawRewards(msg))
            }
            "/cosmos.distribution.v1beta1.SetWithdrawAddr" => {
                let msg = MsgSetWithdrawAddr::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| gears::core::errors::CoreError::DecodeProtobuf(e.to_string()))?;
                Ok(Message::SetWithdrawAddr(msg))
            }
            "/cosmos.distribution.v1beta1.FundCommunityPool" => {
                let msg = MsgFundCommunityPool::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| gears::core::errors::CoreError::DecodeProtobuf(e.to_string()))?;
                Ok(Message::FundCommunityPool(msg))
            }
            _ => Err(gears::core::errors::CoreError::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
