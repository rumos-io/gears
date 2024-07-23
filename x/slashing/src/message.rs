use bytes::Bytes;
use gears::core::any::google::Any;
use gears::core::Protobuf;
use gears::types::address::AccAddress;
use gears::types::tx::TxMessage;
use serde::Serialize;

use crate::MsgUnjail;

#[derive(Debug, Clone, Serialize)]
pub enum Message {
    #[serde(rename = "/cosmos.slashing.v1beta1.Unjail")]
    Unjail(MsgUnjail),
}

impl TxMessage for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match self {
            Message::Unjail(msg) => vec![&msg.from_address],
        }
    }

    fn type_url(&self) -> &'static str {
        match self {
            Message::Unjail(_) => "/cosmos.slashing.v1beta1.Unjail",
        }
    }
}

impl From<Message> for Any {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Unjail(msg) => Any {
                type_url: "/cosmos.slashing.v1beta1.Unjail".to_string(),
                value: msg.encode_vec(),
            },
        }
    }
}

impl TryFrom<Any> for Message {
    type Error = gears::core::errors::CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "/cosmos.slashing.v1beta1.Unjail" => {
                let msg = MsgUnjail::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| gears::core::errors::CoreError::DecodeProtobuf(e.to_string()))?;
                Ok(Message::Unjail(msg))
            }
            _ => Err(gears::core::errors::CoreError::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
