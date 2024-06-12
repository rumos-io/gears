use crate::{CreateValidator, DelegateMsg, RedelegateMsg};
use gears::{
    core::{any::google::Any, Protobuf},
    types::{address::AccAddress, tx::TxMessage},
};
use prost::bytes::Bytes;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "@type")]
#[allow(clippy::large_enum_variant)]
pub enum Message {
    #[serde(rename = "/cosmos.staking.v1beta1.CreateValidator")]
    CreateValidator(CreateValidator),
    #[serde(rename = "/cosmos.staking.v1beta1.Delegate")]
    Delegate(DelegateMsg),
    #[serde(rename = "/cosmos.staking.v1beta1.Redelegate")]
    Redelegate(RedelegateMsg),
}

impl TxMessage for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match &self {
            Message::CreateValidator(msg) => vec![&msg.delegator_address],
            Message::Delegate(msg) => vec![&msg.delegator_address],
            Message::Redelegate(msg) => vec![&msg.delegator_address],
        }
    }

    fn validate_basic(&self) -> Result<(), String> {
        match &self {
            Message::CreateValidator(_) => Ok(()),
            Message::Delegate(_) => Ok(()),
            Message::Redelegate(_) => Ok(()),
        }
    }

    fn type_url(&self) -> &'static str {
        match self {
            Message::CreateValidator(_) => "/cosmos.staking.v1beta1.CreateValidator",
            Message::Delegate(_) => "/cosmos.staking.v1beta1.Delegate",
            Message::Redelegate(_) => "/cosmos.staking.v1beta1.Redelegate",
        }
    }
}

impl From<Message> for Any {
    fn from(msg: Message) -> Self {
        match msg {
            Message::CreateValidator(msg) => Any {
                type_url: "/cosmos.staking.v1beta1.CreateValidator".to_string(),
                value: msg.encode_vec(),
            },
            Message::Delegate(msg) => Any {
                type_url: "/cosmos.staking.v1beta1.Delegate".to_string(),
                value: msg.encode_vec(),
            },
            Message::Redelegate(msg) => Any {
                type_url: "/cosmos.staking.v1beta1.Redelegate".to_string(),
                value: msg.encode_vec(),
            },
        }
    }
}

impl TryFrom<Any> for Message {
    type Error = gears::core::errors::Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "/cosmos.staking.v1beta1.CreateValidator" => {
                let msg = CreateValidator::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| gears::core::errors::Error::DecodeProtobuf(e.to_string()))?;
                Ok(Message::CreateValidator(msg))
            }
            "/cosmos.staking.v1beta1.Delegate" => {
                let msg = DelegateMsg::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| gears::core::errors::Error::DecodeProtobuf(e.to_string()))?;
                Ok(Message::Delegate(msg))
            }
            "/cosmos.staking.v1beta1.Redelegate" => {
                let msg = RedelegateMsg::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| gears::core::errors::Error::DecodeProtobuf(e.to_string()))?;
                Ok(Message::Redelegate(msg))
            }
            _ => Err(gears::core::errors::Error::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
