use gears::{
    core::{address::AccAddress, any::google::Any},
    error::IBC_ENCODE_UNWRAP,
    signing::{
        handler::MetadataGetter,
        renderer::value_renderer::{RenderError, ValueRenderer},
    },
    tendermint::types::proto::Protobuf,
    types::{msg::send::MsgSend, rendering::screen::Screen, tx::TxMessage},
};
use prost::bytes::Bytes;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "@type")]
pub enum Message {
    #[serde(rename = "/cosmos.staking.v1beta1.Todo")]
    Todo(MsgSend),
}

impl ValueRenderer for Message {
    fn format<MG: MetadataGetter>(&self, get_metadata: &MG) -> Result<Vec<Screen>, RenderError> {
        match self {
            Message::Todo(msg) => msg.format(get_metadata),
        }
    }
}

impl TxMessage for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match &self {
            Message::Todo(msg) => vec![&msg.from_address],
        }
    }

    fn validate_basic(&self) -> Result<(), String> {
        match &self {
            Message::Todo(_) => Ok(()),
        }
    }

    fn type_url(&self) -> &'static str {
        match self {
            Message::Todo(_) => "/cosmos.bank.v1beta1.MsgSend",
        }
    }
}

impl From<Message> for Any {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Todo(msg) => Any {
                type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
                value: msg.encode_vec().expect(IBC_ENCODE_UNWRAP),
            },
        }
    }
}

impl TryFrom<Any> for Message {
    type Error = gears::core::errors::Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "/cosmos.bank.v1beta1.MsgSend" => {
                let msg = MsgSend::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| gears::core::errors::Error::DecodeProtobuf(e.to_string()))?;
                Ok(Message::Todo(msg))
            }
            _ => Err(gears::core::errors::Error::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
