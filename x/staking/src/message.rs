use crate::{CreateValidator, DelegateMsg, EditValidator, RedelegateMsg, UndelegateMsg};
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
    #[serde(rename = "/cosmos.staking.v1beta1.MsgCreateValidator")]
    CreateValidator(CreateValidator),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgEditValidator")]
    EditValidator(EditValidator),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgDelegate")]
    Delegate(DelegateMsg),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgBeginRedelegate")]
    Redelegate(RedelegateMsg),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgUndelegate")]
    Undelegate(UndelegateMsg),
}

impl TxMessage for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match &self {
            Message::CreateValidator(msg) => vec![&msg.delegator_address],
            Message::EditValidator(msg) => msg.get_signers(),
            Message::Delegate(msg) => vec![&msg.delegator_address],
            Message::Redelegate(msg) => vec![&msg.delegator_address],
            Message::Undelegate(msg) => vec![&msg.delegator_address],
        }
    }

    fn type_url(&self) -> &'static str {
        match self {
            Message::CreateValidator(_) => CreateValidator::TYPE_URL,
            Message::EditValidator(_) => "/cosmos.staking.v1beta1.MsgEditValidator",
            Message::Delegate(_) => "/cosmos.staking.v1beta1.MsgDelegate",
            Message::Redelegate(_) => "/cosmos.staking.v1beta1.MsgBeginRedelegate",
            Message::Undelegate(_) => "/cosmos.staking.v1beta1.MsgUndelegate",
        }
    }

    fn amino_url(&self) -> &'static str {
        match self {
            Message::Delegate(_) => "cosmos-sdk/MsgDelegate",
            Message::Redelegate(_) => "cosmos-sdk/MsgBeginRedelegate",
            Message::Undelegate(_) => "cosmos-sdk/MsgUndelegate",
            _ => self.type_url(),
        }
    }
}

impl From<Message> for Any {
    fn from(msg: Message) -> Self {
        match msg {
            Message::CreateValidator(msg) => Any {
                type_url: CreateValidator::TYPE_URL.to_string(),
                value: msg.encode_vec(),
            },
            Message::EditValidator(msg) => Any {
                type_url: "/cosmos.staking.v1beta1.MsgEditValidator".to_string(),
                value: msg.encode_vec(),
            },
            Message::Delegate(msg) => Any {
                type_url: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
                value: msg.encode_vec(),
            },
            Message::Redelegate(msg) => Any {
                type_url: "/cosmos.staking.v1beta1.MsgBeginRedelegate".to_string(),
                value: msg.encode_vec(),
            },
            Message::Undelegate(msg) => Any {
                type_url: "/cosmos.staking.v1beta1.MsgUndelegate".to_string(),
                value: msg.encode_vec(),
            },
        }
    }
}

impl TryFrom<Any> for Message {
    type Error = gears::core::errors::CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            CreateValidator::TYPE_URL => {
                let msg = CreateValidator::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| gears::core::errors::CoreError::DecodeProtobuf(e.to_string()))?;
                Ok(Message::CreateValidator(msg))
            }
            "/cosmos.staking.v1beta1.MsgEditValidator" => {
                let msg = EditValidator::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| gears::core::errors::CoreError::DecodeProtobuf(e.to_string()))?;
                Ok(Message::EditValidator(msg))
            }
            "/cosmos.staking.v1beta1.MsgDelegate" => {
                let msg = DelegateMsg::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| gears::core::errors::CoreError::DecodeProtobuf(e.to_string()))?;
                Ok(Message::Delegate(msg))
            }
            "/cosmos.staking.v1beta1.MsgBeginRedelegate" => {
                let msg = RedelegateMsg::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| gears::core::errors::CoreError::DecodeProtobuf(e.to_string()))?;
                Ok(Message::Redelegate(msg))
            }
            "/cosmos.staking.v1beta1.MsgUndelegate" => {
                let msg = UndelegateMsg::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| gears::core::errors::CoreError::DecodeProtobuf(e.to_string()))?;
                Ok(Message::Undelegate(msg))
            }
            _ => Err(gears::core::errors::CoreError::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
