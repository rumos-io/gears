use bytes::Bytes;
use proto_messages::cosmos::ibc_types::protobuf::{Any, Protobuf};
use proto_messages::cosmos::bank::v1beta1::MsgSend;
use proto_types::AccAddress;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "@type")]
pub enum Message {
    #[serde(rename = "/cosmos.bank.v1beta1.MsgSend")]
    Send(MsgSend),
}

impl proto_messages::cosmos::tx::v1beta1::message::Message for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match &self {
            Message::Send(msg) => vec![&msg.from_address],
        }
    }

    fn validate_basic(&self) -> Result<(), String> {
        match &self {
            Message::Send(_) => Ok(()),
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
                value: msg.encode_vec(),
            },
        }
    }
}

impl TryFrom<Any> for Message {
    type Error = proto_messages::Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "/cosmos.bank.v1beta1.MsgSend" => {
                let msg = MsgSend::decode::<Bytes>(value.value.clone().into())?;
                Ok(Message::Send(msg))
            }
            _ => Err(proto_messages::Error::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
