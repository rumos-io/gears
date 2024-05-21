use crate::{CreateValidator, DelegateMsg};
use gears::{
    core::{any::google::Any, Protobuf},
    error::IBC_ENCODE_UNWRAP,
    types::{address::AccAddress, tx::TxMessage},
};
use prost::bytes::Bytes;
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(tag = "@type")]
pub enum Message {
    #[serde(rename = "/cosmos.staking.v1beta1.CreateValidator")]
    CreateValidator(CreateValidator),
    #[serde(rename = "/cosmos.staking.v1beta1.Delegate")]
    Delegate(DelegateMsg),
}

impl TxMessage for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match &self {
            Message::CreateValidator(msg) => vec![&msg.delegator_address],
            Message::Delegate(msg) => vec![&msg.delegator_address],
        }
    }

    fn validate_basic(&self) -> Result<(), String> {
        match &self {
            Message::CreateValidator(_) => Ok(()),
            Message::Delegate(_) => Ok(()),
        }
    }

    fn type_url(&self) -> &'static str {
        match self {
            Message::CreateValidator(_) => "/cosmos.staking.v1beta1.CreateValidator",
            Message::Delegate(_) => "/cosmos.staking.v1beta1.Delegate",
        }
    }
}

impl From<Message> for Any {
    fn from(msg: Message) -> Self {
        match msg {
            Message::CreateValidator(msg) => Any {
                type_url: "/cosmos.staking.v1beta1.CreateValidator".to_string(),
                value: msg.encode_vec().expect(IBC_ENCODE_UNWRAP),
            },
            Message::Delegate(msg) => Any {
                type_url: "/cosmos.staking.v1beta1.Delegate".to_string(),
                value: msg.encode_vec().expect(IBC_ENCODE_UNWRAP),
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
            _ => Err(gears::core::errors::Error::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
