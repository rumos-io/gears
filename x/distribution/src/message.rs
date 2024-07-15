use bytes::Bytes;
use gears::core::any::google::Any;
use gears::core::Protobuf;
use gears::types::address::AccAddress;
use gears::types::tx::TxMessage;
use serde::Serialize;

use crate::MsgWithdrawDelegatorReward;

#[derive(Debug, Clone, Serialize)]
pub enum Message {
    #[serde(rename = "/cosmos.distribution.v1beta1.WithdrawRewards")]
    WithdrawRewards(MsgWithdrawDelegatorReward),
}

impl TxMessage for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match self {
            Message::WithdrawRewards(msg) => vec![&msg.delegator_address],
        }
    }

    fn validate_basic(&self) -> Result<(), String> {
        Ok(())
    }

    fn type_url(&self) -> &'static str {
        match self {
            Message::WithdrawRewards(_) => "/cosmos.distribution.v1beta1.WithdrawRewards",
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
            _ => Err(gears::core::errors::CoreError::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
