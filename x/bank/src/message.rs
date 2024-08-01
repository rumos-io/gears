use bytes::Bytes;
use gears::{
    core::{any::google::Any, errors::CoreError},
    error::IBC_ENCODE_UNWRAP,
    signing::{
        handler::MetadataGetter,
        renderer::value_renderer::{RenderError, ValueRenderer},
    },
    tendermint::types::proto::Protobuf,
    types::{address::AccAddress, msg::send::MsgSend, rendering::screen::Screen, tx::TxMessage},
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "@type")]
pub enum Message {
    #[serde(rename = "/cosmos.bank.v1beta1.MsgSend")]
    Send(MsgSend),
}

impl ValueRenderer for Message {
    fn format<MG: MetadataGetter>(&self, get_metadata: &MG) -> Result<Vec<Screen>, RenderError> {
        match self {
            Message::Send(msg) => msg.format(get_metadata),
        }
    }
}

impl TxMessage for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match &self {
            Message::Send(msg) => vec![&msg.from_address],
        }
    }

    fn type_url(&self) -> &'static str {
        match self {
            Message::Send(_) => "/cosmos.bank.v1beta1.MsgSend",
        }
    }
}

impl From<Message> for Any {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Send(msg) => Any {
                type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
                value: msg.encode_vec().expect(IBC_ENCODE_UNWRAP),
            },
        }
    }
}

impl TryFrom<Any> for Message {
    type Error = gears::core::errors::CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "/cosmos.bank.v1beta1.MsgSend" => {
                let msg = MsgSend::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))?;
                Ok(Message::Send(msg))
            }
            _ => Err(CoreError::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
